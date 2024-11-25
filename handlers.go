package main

import (
	"bufio"
	"encoding/json"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

func handleStreamResponse(c *gin.Context, req ChatRequest) {
	c.Header("Content-Type", "text/event-stream")
	c.Header("Cache-Control", "no-cache")
	c.Header("Connection", "keep-alive")
	
	responseId := "chatcmpl-" + uuid.New().String()
	
	// 准备请求数据
	messages := formatMessages(req.Messages)
	hexData := stringToHex(messages, req.Model)
	
	// 发送请求到 Cursor API
	resp, err := sendToCursorAPI(c, hexData)
	if err != nil {
		c.SSEvent("error", gin.H{"error": "Internal server error"})
		return
	}
	defer resp.Body.Close()

	reader := bufio.NewReader(resp.Body)
	for {
		chunk, err := reader.ReadBytes('\n')
		if err == io.EOF {
			break
		}
		if err != nil {
			continue
		}

		text := processChunk(chunk)
		if text == "" {
			continue
		}

		streamResp := StreamResponse{
			ID:      responseId,
			Object:  "chat.completion.chunk",
			Created: time.Now().Unix(),
			Model:   req.Model,
			Choices: []struct {
				Index int `json:"index"`
				Delta struct {
					Content string `json:"content"`
				} `json:"delta"`
			}{{
				Index: 0,
				Delta: struct {
					Content string `json:"content"`
				}{
					Content: text,
				},
			}},
		}

		data, _ := json.Marshal(streamResp)
		c.SSEvent("message", string(data))
		c.Writer.Flush()
	}

	c.SSEvent("message", "[DONE]")
}

func handleNormalResponse(c *gin.Context, req ChatRequest) {
	messages := formatMessages(req.Messages)
	hexData := stringToHex(messages, req.Model)

	resp, err := sendToCursorAPI(c, hexData)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Internal server error"})
		return
	}
	defer resp.Body.Close()

	var fullText strings.Builder
	reader := bufio.NewReader(resp.Body)
	for {
		chunk, err := reader.ReadBytes('\n')
		if err == io.EOF {
			break
		}
		if err != nil {
			continue
		}

		text := processChunk(chunk)
		fullText.WriteString(text)
	}

	// 处理响应文本
	text := fullText.String()
	text = strings.TrimSpace(strings.TrimPrefix(text, "<|END_USER|>"))

	response := ChatResponse{
		ID:      "chatcmpl-" + uuid.New().String(),
		Object:  "chat.completion",
		Created: time.Now().Unix(),
		Model:   req.Model,
		Choices: []struct {
			Index        int    `json:"index"`
			Message      struct {
				Role    string `json:"role"`
				Content string `json:"content"`
			} `json:"message"`
			FinishReason string `json:"finish_reason"`
		}{{
			Index: 0,
			Message: struct {
				Role    string `json:"role"`
				Content string `json:"content"`
			}{
				Role:    "assistant",
				Content: text,
			},
			FinishReason: "stop",
		}},
		Usage: struct {
			PromptTokens     int `json:"prompt_tokens"`
			CompletionTokens int `json:"completion_tokens"`
			TotalTokens      int `json:"total_tokens"`
		}{},
	}

	c.JSON(http.StatusOK, response)
} 