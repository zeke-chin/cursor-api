module.exports = {
  apps: [{
    name: 'cursor-api', // 应用程序名称
    script: 'src/index.js', // 启动脚本路径
    instances: 1, // 实例数量
    autorestart: true, // 自动重启
    watch: false, // 文件变化监控
    max_memory_restart: '1G', // 内存限制重启
    log_date_format: 'YYYY-MM-DD HH:mm:ss', // 日志时间格式
    error_file: 'logs/error.log', // 错误日志路径
    out_file: 'logs/out.log', // 输出日志路径
    log_file: 'logs/combined.log', // 组合日志路径
    merge_logs: true, // 合并集群模式的日志
    rotate_interval: '1d' // 日志轮转间隔
  }]
}
