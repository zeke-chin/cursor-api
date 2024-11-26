import json

from fastapi import FastAPI, Request, Response, HTTPException
from fastapi.responses import StreamingResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import uuid
import httpx
import os
from dotenv import load_dotenv
import time
import re

# 加载环境变量
load_dotenv()

app = FastAPI()

# 添加CORS中间件
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# 定义请求模型
class Message(BaseModel):
    role: str
    content: str


class ChatRequest(BaseModel):
    model: str
    messages: List[Message]
    stream: bool = False


def string_to_hex(text: str, model_name: str) -> bytes:
    """将文本转换为特定格式的十六进制数据"""
    # 将输入文本转换为UTF-8字节
    text_bytes = text.encode('utf-8')
    text_length = len(text_bytes)

    # 固定常量
    FIXED_HEADER = 2
    SEPARATOR = 1
    FIXED_SUFFIX_LENGTH = 0xA3 + len(model_name)

    # 计算第一个长度字段
    if text_length < 128:
        text_length_field1 = format(text_length, '02x')
        text_length_field_size1 = 1
    else:
        low_byte1 = (text_length & 0x7F) | 0x80
        high_byte1 = (text_length >> 7) & 0xFF
        text_length_field1 = format(low_byte1, '02x') + format(high_byte1, '02x')
        text_length_field_size1 = 2

    # 计算基础长度字段
    base_length = text_length + 0x2A
    if base_length < 128:
        text_length_field = format(base_length, '02x')
        text_length_field_size = 1
    else:
        low_byte = (base_length & 0x7F) | 0x80
        high_byte = (base_length >> 7) & 0xFF
        text_length_field = format(low_byte, '02x') + format(high_byte, '02x')
        text_length_field_size = 2

    # 计算总消息长度
    message_total_length = (FIXED_HEADER + text_length_field_size + SEPARATOR +
                            text_length_field_size1 + text_length + FIXED_SUFFIX_LENGTH)

    # 构造十六进制字符串
    model_name_bytes = model_name.encode('utf-8')
    model_name_length_hex = format(len(model_name_bytes), '02X')
    model_name_hex = model_name_bytes.hex().upper()

    hex_string = (
        f"{message_total_length:010x}"
        "12"
        f"{text_length_field}"
        "0A"
        f"{text_length_field1}"
        f"{text_bytes.hex()}"
        "10016A2432343163636435662D393162612D343131382D393239612D3936626330313631626432612"
        "2002A132F643A2F6964656150726F2F656475626F73733A1E0A"
        f"{model_name_length_hex}"
        f"{model_name_hex}"
        "22004A"
        "2461383761396133342D323164642D343863372D623434662D616636633365636536663765"
        "680070007A2436393337376535612D386332642D343835342D623564392D653062623232336163303061"
        "800101B00100C00100E00100E80100"
    ).upper()

    return bytes.fromhex(hex_string)


def chunk_to_utf8_string(chunk: bytes) -> str:
    """将二进制chunk转换为UTF-8字符串"""
    if not chunk or len(chunk) < 2:
        return ''

    if chunk[0] in [0x01, 0x02] or (chunk[0] == 0x60 and chunk[1] == 0x0C):
        return ''

    # 记录原始chunk的十六进制（调试用）
    print(f"chunk length: {len(chunk)}")
    # print(f"chunk hex: {chunk.hex()}")

    try:
        # 去掉0x0A之前的所有字节
        try:
            chunk = chunk[chunk.index(0x0A) + 1:]
        except ValueError:
            pass

        filtered_chunk = bytearray()
        i = 0
        while i < len(chunk):
            # 检查是否有连续的0x00
            if i + 4 <= len(chunk) and all(chunk[j] == 0x00 for j in range(i, i + 4)):
                i += 4
                while i < len(chunk) and chunk[i] <= 0x0F:
                    i += 1
                continue

            if chunk[i] == 0x0C:
                i += 1
                while i < len(chunk) and chunk[i] == 0x0A:
                    i += 1
            else:
                filtered_chunk.append(chunk[i])
                i += 1

        # 过滤掉特定字节
        filtered_chunk = bytes(b for b in filtered_chunk
                               if b != 0x00 and b != 0x0C)

        if not filtered_chunk:
            return ''

        result = filtered_chunk.decode('utf-8', errors='ignore').strip()
        # print(f"decoded result: {result}")  # 调试输出
        return result

    except Exception as e:
        print(f"Error in chunk_to_utf8_string: {str(e)}")
        return ''


async def process_stream(chunks, ):
    """处理流式响应"""
    response_id = f"chatcmpl-{str(uuid.uuid4())}"

    # 先将所有chunks读取到列表中
    # chunks = []
    # async for chunk in response.aiter_raw():
    #     chunks.append(chunk)

    # 然后处理保存的chunks
    for chunk in chunks:
        text = chunk_to_utf8_string(chunk)
        if text:
            # 清理文本
            text = text.strip()
            if "<|END_USER|>" in text:
                text = text.split("<|END_USER|>")[-1].strip()
            if text and text[0].isalpha():
                text = text[1:].strip()
            text = re.sub(r"[\x00-\x1F\x7F]", "", text)

            if text:  # 确保清理后的文本不为空
                data_body = {
                    "id": response_id,
                    "object": "chat.completion.chunk",
                    "created": int(time.time()),
                    "choices": [{
                        "index": 0,
                        "delta": {
                            "content": text
                        }
                    }]
                }
                yield f"data: {json.dumps(data_body, ensure_ascii=False)}\n\n"
                # yield "data: {\n"
                # yield f'    "id": "{response_id}",\n'
                # yield '    "object": "chat.completion.chunk",\n'
                # yield f'    "created": {int(time.time())},\n'
                # yield '    "choices": [{\n'
                # yield '        "index": 0,\n'
                # yield '        "delta": {\n'
                # yield f'            "content": "{text}"\n'
                # yield "        }\n"
                # yield "    }]\n"
                # yield "}\n\n"

    yield "data: [DONE]\n\n"


@app.post("/v1/chat/completions")
async def chat_completions(request: Request, chat_request: ChatRequest):
    # 验证o1模型不支持流式输出
    if chat_request.model.startswith('o1-') and chat_request.stream:
        raise HTTPException(
            status_code=400,
            detail="Model not supported stream"
        )

    # 获取并处理认证令牌
    auth_header = request.headers.get('authorization', '')
    if not auth_header.startswith('Bearer '):
        raise HTTPException(
            status_code=401,
            detail="Invalid authorization header"
        )

    auth_token = auth_header.replace('Bearer ', '')
    if not auth_token:
        raise HTTPException(
            status_code=401,
            detail="Missing authorization token"
        )

    # 处理多个密钥
    keys = [key.strip() for key in auth_token.split(',')]
    if keys:
        auth_token = keys[0]  # 使用第一个密钥

    if '%3A%3A' in auth_token:
        auth_token = auth_token.split('%3A%3A')[1]

    # 格式化消息
    formatted_messages = "\n".join(
        f"{msg.role}:{msg.content}" for msg in chat_request.messages
    )

    # 生成请求数据
    hex_data = string_to_hex(formatted_messages, chat_request.model)

    # 准备请求头
    headers = {
        'Content-Type': 'application/connect+proto',
        'Authorization': f'Bearer {auth_token}',
        'Connect-Accept-Encoding': 'gzip,br',
        'Connect-Protocol-Version': '1',
        'User-Agent': 'connect-es/1.4.0',
        'X-Amzn-Trace-Id': f'Root={str(uuid.uuid4())}',
        'X-Cursor-Checksum': 'zo6Qjequ9b9734d1f13c3438ba25ea31ac93d9287248b9d30434934e9fcbfa6b3b22029e/7e4af391f67188693b722eff0090e8e6608bca8fa320ef20a0ccb5d7d62dfdef',
        'X-Cursor-Client-Version': '0.42.3',
        'X-Cursor-Timezone': 'Asia/Shanghai',
        'X-Ghost-Mode': 'false',
        'X-Request-Id': str(uuid.uuid4()),
        'Host': 'api2.cursor.sh'
    }

    async with httpx.AsyncClient(timeout=httpx.Timeout(300.0)) as client:
        try:
            # 使用 stream=True 参数
            # 打印 headers 和 二进制 data
            print(f"headers: {headers}")
            print(hex_data)
            async with client.stream(
                    'POST',
                    'https://api2.cursor.sh/aiserver.v1.AiService/StreamChat',
                    headers=headers,
                    content=hex_data,
                    timeout=None
            ) as response:
                if chat_request.stream:
                    chunks = []
                    async for chunk in response.aiter_raw():
                        chunks.append(chunk)
                    return StreamingResponse(
                        process_stream(chunks),
                        media_type="text/event-stream"
                    )
                else:
                    # 非流式响应处理
                    text = ''
                    async for chunk in response.aiter_raw():
                        # print('chunk:', chunk.hex())
                        print('chunk length:', len(chunk))

                        res = chunk_to_utf8_string(chunk)
                        # print('res:', res)
                        if res:
                            text += res

                    # 清理响应文本
                    import re
                    text = re.sub(r'^.*<\|END_USER\|>', '', text, flags=re.DOTALL)
                    text = re.sub(r'^\n[a-zA-Z]?', '', text).strip()
                    text = re.sub(r'[\x00-\x1F\x7F]', '', text)

                    return {
                        "id": f"chatcmpl-{str(uuid.uuid4())}",
                        "object": "chat.completion",
                        "created": int(time.time()),
                        "model": chat_request.model,
                        "choices": [{
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": text
                            },
                            "finish_reason": "stop"
                        }],
                        "usage": {
                            "prompt_tokens": 0,
                            "completion_tokens": 0,
                            "total_tokens": 0
                        }
                    }

        except Exception as e:
            print(f"Error: {str(e)}")
            raise HTTPException(
                status_code=500,
                detail="Internal server error"
            )


@app.post("/models")
async def models():
    return {
        "object": "list",
        "data": [
            {
                "id": "claude-3-5-sonnet-20241022",
                "object": "model",
                "created": 1713744000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3-opus",
                "object": "model",
                "created": 1709251200,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3.5-haiku",
                "object": "model",
                "created": 1711929600,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-3.5-sonnet",
                "object": "model",
                "created": 1711929600,
                "owned_by": "anthropic"
            },
            {
                "id": "cursor-small",
                "object": "model",
                "created": 1712534400,
                "owned_by": "cursor"
            },
            {
                "id": "gpt-3.5-turbo",
                "object": "model",
                "created": 1677649200,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4",
                "object": "model",
                "created": 1687392000,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4-turbo-2024-04-09",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o-mini",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "o1-mini",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            },
            {
                "id": "o1-preview",
                "object": "model",
                "created": 1712620800,
                "owned_by": "openai"
            }
        ]
    }

if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", "3001"))
    uvicorn.run("main:app", host="0.0.0.0", port=port, reload=True, timeout_keep_alive=30)
