package main

import (
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
)

func main() {
	if err := godotenv.Load(); err != nil {
		log.Println("Warning: Error loading .env file")
	}

	r := gin.Default()
	r.POST("/v1/chat/completions", handleChat)

	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}
	
	log.Printf("服务器运行在端口 %s\n", port)
	r.Run(":" + port)
}

func handleChat(c *gin.Context) {
	var req ChatRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 检查是否为 o1 开头的模型且请求流式输出
	if strings.HasPrefix(req.Model, "o1-") && req.Stream {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Model not supported stream"})
		return
	}

	// 获取并处理认证token
	authHeader := c.GetHeader("Authorization")
	authToken := strings.TrimPrefix(authHeader, "Bearer ")
	
	if authToken == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Authorization is required",
		})
		return
	}

	// 处理消息
	if len(req.Messages) == 0 {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Messages should be a non-empty array",
		})
		return
	}

	// 处理流式请求
	if req.Stream {
		handleStreamResponse(c, req)
		return
	}

	// 处理非流式请求
	handleNormalResponse(c, req)
} 