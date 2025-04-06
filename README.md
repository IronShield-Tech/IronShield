# Ironshield

**Under a L7 DDoS attack? Sophisticated attackers spamming your API? If you need real protection but can't yet justify Cloudflare Enterprise prices, Ironshield is your defense.**

## The Problem

You've built something valuable, and now you're under attack:

- Your API endpoints are hammered with thousands of requests per second
- Sophisticated attackers bypass basic rate limiting
- L7 DDoS attacks target your application logic, not just your bandwidth
- Commercial solutions like Cloudflare Enterprise cost thousands monthly
- **You have Cloudflare Enterprise-level problems but not Cloudflare Enterprise-level money**
- Each minute of downtime costs you users and reputation

Meanwhile, existing open-source solutions are often:
- Outdated or unmaintained (PoW-Shield maintainer can't maintain if he's in jail)
- Written in inefficient languages that consume too many resources
- Difficult to configure and deploy
- Unable to handle modern attack vectors

## Enter Ironshield

Ironshield is a high-performance, Rust-based DDoS protection layer designed specifically for crypto projects, APIs, and self-hosted services that need professional-grade protection without enterprise costs.

Built as a modern rewrite of the conceptual foundation laid by POW-Shield, Ironshield leverages Rust's speed, safety, and concurrency to provide DDoS protection that's orders of magnitude more efficient.

### Key Features

- **Proof of Work Challenge**: Forces attackers to expend significant computational resources, making DDoS attacks economically impractical
- **Blazing Fast Proxy**: Written in Rust for minimal overhead and maximum throughput
- **Smart Rate Limiting**: Sophisticated algorithms that recognize and allow legitimate traffic patterns
- **Advanced WAF Protection**: Detects and blocks common attack patterns before they reach your application
- **Distributed Architecture**: Scales horizontally across your infrastructure
- **Privacy Preserving**: Keeps your traffic on your infrastructure, not routed through third parties
- **Low Resource Utilization**: Tiny memory footprint compared to Node.js-based alternatives

## Who Is This For?

- **Crypto exchanges & DeFi platforms** facing targeted attacks
- **Web3 infrastructure providers** protecting critical APIs
- **NFT projects** during high-demand mints
- **Blockchain explorers** and similar high-value targets
- **Privacy-focused services** that can't use centralized protection
- **Self-hosted applications** requiring professional protection
- **Small to medium businesses** with budget constraints
- **Teams demanding full control** over their stack, free from proprietary cloud dependencies

## How It Works

Ironshield sits as a reverse proxy in front of your application:

1. When a client makes a request, Ironshield issues a proof-of-work challenge
2. The client's browser solves the computational puzzle via JavaScript
3. Once verified, the client receives a time-limited token
4. Subsequent requests with valid tokens are proxied to your application
5. Malicious requests never reach your origin server

The computational cost is negligible for legitimate users visiting your site normally, but prohibitively expensive for attackers attempting to generate thousands of requests per second.
