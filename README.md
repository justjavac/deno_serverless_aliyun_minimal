# 阿里云函数计算 Deno FC Runtime

Deno + Serverless = Awesome

## 背景

两年前（2020年）我曾经写过一个项目，给阿里云的 Severless 开发了可运行 Deno 的 Custom Runtime:
[deno_serverless_aliyun](https://github.com/justjavac/deno_serverless_aliyun)。

今天我重新从源码编译了 deno_core，并且把
[src/fc_runtime.js](fc-custom-rust-event/bootstrap/src/fc_runtime.js) 编译为了 V8
Snapshot 进一步缩短冷启动时间。

**注意⚠️：本项目不是一个完整的 Deno Custom Runtime⚠️**

## 前置工具

- [Serverless Devs](https://docs.serverless-devs.com/serverless-devs/install)
- Docker（如果你使用 linux 系统，可以不安装 Docker）

## 配置

安装 Serverless Devs 成功后运行
[`s config`](https://docs.serverless-devs.com/serverless-devs/command/config) 配置
Account ID、Access Key ID、Secret Access Key、Default Region Name。

克隆（或下载）本仓库:

```shell
git clone git@github.com:justjavac/deno_serverless_aliyun_minimal.git
```

## 部署

```shell
cd deno_serverless_aliyun_minimal
make deploy
```

自定义部署:

1. 使用 Rust 编译 bootstrap。

   ```shell
   cd bootstrap
   cargo build --release
   ```

   非 linux 系统使用 Docker 构建：

   ```shell
   make build
   ```

2. 构建 Deno Serverless 运行环境：

   ```shell
   s build
   ```

3. 部署

   ```shell
   s deploy
   ```

## 测试

部署完成后我们可以测试刚才的函数。

```shell
s invoke --event "Hello World"
```

控制台输出:

```plain
...
FC Invoke Result:
Hello World
```

第一次运行时函数需要冷启，会稍微有点慢。

对比之前的 Deno + JS:

```plain
Duration: 9.27 ms, Billed Duration: 12 ms, Memory Size: 512 MB, Max Memory Used: 41.50 MB
Duration: 1.66 ms, Billed Duration: 2 ms, Memory Size: 512 MB, Max Memory Used: 41.62 MB
Duration: 1.41 ms, Billed Duration: 2 ms, Memory Size: 512 MB, Max Memory Used: 41.87 MB
Duration: 1.50 ms, Billed Duration: 2 ms, Memory Size: 512 MB, Max Memory Used: 42.25 MB
Duration: 1.41 ms, Billed Duration: 2 ms, Memory Size: 512 MB, Max Memory Used: 42.50 MB
```

Deno_core + V8 Snapshot 的启动时间:

```plain
Duration: 1.21 ms, Billed Duration: 2 ms, Memory Size: 512 MB, Max Memory Used: 18.02 MB
Duration: 0.78 ms, Billed Duration: 1 ms, Memory Size: 512 MB, Max Memory Used: 22.96 MB
Duration: 0.67 ms, Billed Duration: 1 ms, Memory Size: 512 MB, Max Memory Used: 23.08 MB
Duration: 0.73 ms, Billed Duration: 1 ms, Memory Size: 512 MB, Max Memory Used: 22.64 MB
Duration: 0.67 ms, Billed Duration: 1 ms, Memory Size: 512 MB, Max Memory Used: 23.12 MB
```

**注意⚠️：本项目不是一个完整的 Deno Custom Runtime⚠️**
