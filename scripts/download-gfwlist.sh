#!/bin/bash
# 下载并解码 GFWList

echo "正在下载 GFWList..."

# 下载 base64 编码的 GFWList
curl -L "https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt" -o gfwlist.b64

if [ $? -ne 0 ]; then
    echo "下载失败，尝试使用镜像..."
    curl -L "https://cdn.jsdelivr.net/gh/gfwlist/gfwlist/gfwlist.txt" -o gfwlist.b64
fi

if [ -f gfwlist.b64 ]; then
    echo "正在解码 GFWList..."
    base64 -d gfwlist.b64 > gfwlist.txt
    
    if [ $? -eq 0 ]; then
        echo "GFWList 下载成功！已保存到 gfwlist.txt"
        rm gfwlist.b64
        
        # 显示规则数量
        rules=$(grep -v '^!' gfwlist.txt | grep -v '^$' | grep -v '^\[' | wc -l)
        echo "包含 $rules 条规则"
    else
        echo "解码失败"
        exit 1
    fi
else
    echo "下载失败"
    exit 1
fi

