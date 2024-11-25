package api

import (
	"github.com/gin-gonic/gin"
)

func NewServer() *gin.Engine {
	r := gin.Default()
	setupRoutes(r)
	return r
}

func setupRoutes(r *gin.Engine) {
	r.POST("/v1/chat/completions", handleChat)
} 