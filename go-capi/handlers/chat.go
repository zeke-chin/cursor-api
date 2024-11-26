package handlers

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
	"bufio"
	"encoding/json"
	"unicode"
	"regexp"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"go-capi/models"
	"go-capi/utils"
)

func ChatCompletions(c *gin.Context) {
	var chatRequest models.ChatRequest
	if err := c.ShouldBindJSON(&chatRequest); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 验证o1模型不支持流式输出
	if strings.HasPrefix(chatRequest.Model, "o1-") && chatRequest.Stream {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Model not supported stream"})
		return
	}

	// 获取并处理认证令牌
	authHeader := c.GetHeader("Authorization")
	if !strings.HasPrefix(authHeader, "Bearer ") {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid authorization header"})
		return
	}

	authToken := strings.TrimPrefix(authHeader, "Bearer ")
	if authToken == "" {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Missing authorization token"})
		return
	}

	// 处理多个密钥
	keys := strings.Split(authToken, ",")
	if len(keys) > 0 {
		authToken = strings.TrimSpace(keys[0])
	}

	if strings.Contains(authToken, "%3A%3A") {
		parts := strings.Split(authToken, "%3A%3A")
		authToken = parts[1]
	}

	// 格式化消息
	var messages []string
	for _, msg := range chatRequest.Messages {
		messages = append(messages, fmt.Sprintf("%s:%s", msg.Role, msg.Content))
	}
	formattedMessages := strings.Join(messages, "\n")

	// 生成请求数据
	hexData, err := utils.StringToHex(formattedMessages, chatRequest.Model)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// 准备请求
	client := &http.Client{Timeout: 300 * time.Second}
	req, err := http.NewRequest("POST", "https://api2.cursor.sh/aiserver.v1.AiService/StreamChat", bytes.NewReader(hexData))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// 设置请求头
	req.Header.Set("Content-Type", "application/connect+proto")
	req.Header.Set("Authorization", "Bearer "+authToken)
	req.Header.Set("Connect-Accept-Encoding", "gzip,br")
	req.Header.Set("Connect-Protocol-Version", "1")
	req.Header.Set("User-Agent", "connect-es/1.4.0")
	req.Header.Set("X-Amzn-Trace-Id", "Root="+uuid.New().String())
	req.Header.Set("X-Cursor-Checksum", "zo6Qjequ9b9734d1f13c3438ba25ea31ac93d9287248b9d30434934e9fcbfa6b3b22029e/7e4af391f67188693b722eff0090e8e6608bca8fa320ef20a0ccb5d7d62dfdef")
	req.Header.Set("X-Cursor-Client-Version", "0.42.3")
	req.Header.Set("X-Cursor-Timezone", "Asia/Shanghai")
	req.Header.Set("X-Ghost-Mode", "false")
	req.Header.Set("X-Request-Id", uuid.New().String())
	req.Header.Set("Host", "api2.cursor.sh")
	// ... 设置其他请求头


	// 打印 请求头和请求体
	fmt.Printf("\nRequest Headers: %v\n", req.Header)
	fmt.Printf("\nRequest Body: %x\n", hexData)

	resp, err := client.Do(req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	defer resp.Body.Close()

	if chatRequest.Stream {
		c.Header("Content-Type", "text/event-stream")
		c.Header("Cache-Control", "no-cache")
		c.Header("Connection", "keep-alive")
		
		chunks := make([][]byte, 0)
		reader := bufio.NewReader(resp.Body)
		
		for {
			chunk, err := reader.ReadBytes('\n')
			if err == io.EOF {
				break
			}
			if err != nil {
				c.SSEvent("error", gin.H{"error": err.Error()})
				return
			}
			chunks = append(chunks, chunk)
		}

		responseID := "chatcmpl-" + uuid.New().String()
		c.Stream(func(w io.Writer) bool {
			for _, chunk := range chunks {
				text := chunkToUTF8String(chunk)
				if text == "" {
					continue
				}

				// 清理文本
				text = strings.TrimSpace(text)
				if strings.Contains(text, "<|END_USER|>") {
					parts := strings.Split(text, "<|END_USER|>")
					text = strings.TrimSpace(parts[len(parts)-1])
				}
				if len(text) > 0 && unicode.IsLetter(rune(text[0])) {
					text = strings.TrimSpace(text[1:])
				}
				text = cleanControlChars(text)

				if text != "" {
					dataBody := map[string]interface{}{
						"id":      responseID,
						"object":  "chat.completion.chunk",
						"created": time.Now().Unix(),
						"choices": []map[string]interface{}{
							{
								"index": 0,
								"delta": map[string]string{
									"content": text,
								},
							},
						},
					}
					
					jsonData, _ := json.Marshal(dataBody)
					c.SSEvent("", string(jsonData))
					w.(http.Flusher).Flush()
				}
			}
			
			c.SSEvent("", "[DONE]")
			return false
		})
	} else {
		// 非流式响应处理
		reader := bufio.NewReader(resp.Body)
		var allText string
		
		for {
			chunk, err := reader.ReadBytes('\n')
			if err == io.EOF {
				break
			}
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			
			text := utils.ChunkToUTF8String(chunk)
			if text != "" {
				allText += text
			}
		}
	
		// 清理响应文本
		allText = cleanResponseText(allText)
		

		response := models.ChatResponse{
			ID:      "chatcmpl-" + uuid.New().String(),
			Object:  "chat.completion",
			Created: time.Now().Unix(),
			Model:   chatRequest.Model,
			Choices: []models.Choice{
				{
					Index: 0,
					Message: &models.Message{
						Role:    "assistant",
						Content: allText,
					},
					FinishReason: "stop",
				},
			},
			Usage: &models.Usage{
				PromptTokens:     0,
				CompletionTokens: 0,
				TotalTokens:      0,
			},
		}

		c.JSON(http.StatusOK, response)
	}
}

// 辅助函数
func chunkToUTF8String(chunk []byte) string {
	// 实现从二进制chunk转换到UTF8字符串的逻辑
	return string(chunk)
}

func cleanControlChars(text string) string {
	return regexp.MustCompile(`[\x00-\x1F\x7F]`).ReplaceAllString(text, "")
}

func cleanResponseText(text string) string {
	// 移除END_USER之前的所有内容
	re := regexp.MustCompile(`(?s)^.*<\|END_USER\|>`)
	text = re.ReplaceAllString(text, "")
	
	// 移除开头的换行和单个字母
	text = regexp.MustCompile(`^\n[a-zA-Z]?`).ReplaceAllString(text, "")
	text = strings.TrimSpace(text)
	
	// 清理控制字符
	text = cleanControlChars(text)
	
	return text
}

func GetModels(c *gin.Context) {
	response := models.ModelsResponse{
		Object: "list",
		Data: []models.ModelData{
			{ID: "claude-3-5-sonnet-20241022", Object: "model", Created: 1713744000, OwnedBy: "anthropic"},
			{ID: "claude-3-opus", Object: "model", Created: 1709251200, OwnedBy: "anthropic"},
			{ID: "claude-3.5-haiku", Object: "model", Created: 1711929600, OwnedBy: "anthropic"},
			{ID: "claude-3.5-sonnet", Object: "model", Created: 1711929600, OwnedBy: "anthropic"},
			{ID: "cursor-small", Object: "model", Created: 1712534400, OwnedBy: "cursor"},
			{ID: "gpt-3.5-turbo", Object: "model", Created: 1677649200, OwnedBy: "openai"},
			{ID: "gpt-4", Object: "model", Created: 1687392000, OwnedBy: "openai"},
			{ID: "gpt-4-turbo-2024-04-09", Object: "model", Created: 1712620800, OwnedBy: "openai"},
			{ID: "gpt-4o", Object: "model", Created: 1712620800, OwnedBy: "openai"},
			{ID: "gpt-4o-mini", Object: "model", Created: 1712620800, OwnedBy: "openai"},
			{ID: "o1-mini", Object: "model", Created: 1712620800, OwnedBy: "openai"},
			{ID: "o1-preview", Object: "model", Created: 1712620800, OwnedBy: "openai"},
		},
	}
	c.JSON(http.StatusOK, response)
}