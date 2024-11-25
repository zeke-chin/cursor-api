package main

import (
	"bytes"
	"fmt"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

func formatMessages(messages []Message) string {
	var formatted []string
	for _, msg := range messages {
		formatted = append(formatted, fmt.Sprintf("%s:%s", msg.Role, msg.Content))
	}
	return strings.Join(formatted, "\n")
}

func sendToCursorAPI(c *gin.Context, hexData []byte) (*http.Response, error) {
	req, err := http.NewRequest("POST", "https://api2.cursor.sh/aiserver.v1.AiService/StreamChat", bytes.NewReader(hexData))
	if err != nil {
		return nil, err
	}

	// 获取认证token
	authToken := strings.TrimPrefix(c.GetHeader("Authorization"), "Bearer ")
	if strings.Contains(authToken, "%3A%3A") {
		authToken = strings.Split(authToken, "%3A%3A")[1]
	}

	// 设置请求头
	req.Header.Set("Content-Type", "application/connect+proto")
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", authToken))
	req.Header.Set("Connect-Accept-Encoding", "gzip,br")
	req.Header.Set("Connect-Protocol-Version", "1")
	req.Header.Set("User-Agent", "connect-es/1.4.0")
	req.Header.Set("X-Amzn-Trace-Id", fmt.Sprintf("Root=%s", uuid.New().String()))
	req.Header.Set("X-Cursor-Checksum", "zo6Qjequ9b9734d1f13c3438ba25ea31ac93d9287248b9d30434934e9fcbfa6b3b22029e/7e4af391f67188693b722eff0090e8e6608bca8fa320ef20a0ccb5d7d62dfdef")
	req.Header.Set("X-Cursor-Client-Version", "0.42.3")
	req.Header.Set("X-Cursor-Timezone", "Asia/Shanghai")
	req.Header.Set("X-Ghost-Mode", "false")
	req.Header.Set("X-Request-Id", uuid.New().String())
	req.Header.Set("Host", "api2.cursor.sh")

	client := &http.Client{}
	return client.Do(req)
} 