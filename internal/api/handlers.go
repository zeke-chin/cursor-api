package api

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"cursor-api-proxy/internal/models"
	"cursor-api-proxy/internal/service"
)

func handleChat(c *gin.Context) {
	var req models.ChatRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 检查是否为 o1 开头的模型且请求流式输出
	if strings.HasPrefix(req.Model, "o1-") && req.Stream {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Model not supported stream"})
		return
	}

	// ... 其他处理逻辑 ...
	
	if req.Stream {
		service.HandleStreamResponse(c, req)
		return
	}
	
	service.HandleNormalResponse(c, req)
} 