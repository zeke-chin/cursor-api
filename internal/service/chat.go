package service

import (
	"github.com/gin-gonic/gin"
	"cursor-api-proxy/internal/models"
	"cursor-api-proxy/internal/utils"
)

func HandleStreamResponse(c *gin.Context, req models.ChatRequest) {
	// 从 handlers.go 移动流式响应处理逻辑到这里
}

func HandleNormalResponse(c *gin.Context, req models.ChatRequest) {
	// 从 handlers.go 移动普通响应处理逻辑到这里
} 