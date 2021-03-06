## 简介

nf 是一个网络穿透代理程序。提供穿透及统一隧道功能。

## 支持功能

1. 支持 ipv4,ipv6 的 tcp 网络穿透
2. 支持指定跳转地址链路
3. 转发数据支持 rc4, aes 加解密算法

例如

> tcp 请求 -> node_server -> node_server -> ... -> node_server -> 目标地址

## 例子

```shell script
# 作为跳板节点监听本地8080端口，并接收其他节点转发的请求 。
nf -l 127.0.0.1:8080
# 作为数据转发节点，监听本地8080并转发到8081端口上。
nf -l 127.0.0.1:8080 -L 127.0.0.1:8081
# 本地监听8080 端口，并将数据通过-L 指定的链路顺序跳转后转发到8081端口上
nf -l 127.0.0.1:80801 -L 127.0.0.1:8090,localhost:8091,[::1]:8092

# param
-c aes -k 1234567890qweewq32rtyuio432Tadfg
-c rc4 -k 123456
```
