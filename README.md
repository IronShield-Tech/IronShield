# IronShield

## Enterprise-Level DDoS Protection at Startup-Friendly Prices

### What is IronShield?

IronShield is a powerful, Rust-based Layer 7 (L7) DDoS and bot protection solution specifically built for APIs, crypto projects, Web3 platforms, self-hosted applications, and startups that demand enterprise-level security but lack enterprise-level budgets. This project originated from the need for effective DDoS protection that respects user privacy, as many traditional security solutions inadvertently block legitimate users accessing services via privacy tools like VPNs and Tor. Primarily designed as a solution to be effortlessly deployed on Cloudflare®'s infrastructure, IronShield provides robust protection without relying exclusively on Cloudflare®'s built-in bot detection mechanisms, ensuring compatibility with privacy-focused networks like Tor and VPNs, essential for cryptocurrency and privacy-first services.

IronShield delivers top-tier protection without the heavy costs associated with traditional enterprise solutions like Cloudflare® Enterprise and provides a scalable infrastructure solution instead of the common janky self-rolled options currently prevalent.

### The Problem

You've built an innovative, valuable service—but now you're under attack:

- Your APIs are overwhelmed by thousands of malicious requests per second.
- Attackers easily bypass basic rate limiting and firewalls.
- You're suffering from Layer 7 DDoS attacks targeting your application's logic, not just network bandwidth.
- Cloudflare® Enterprise protection is prohibitively expensive (often thousands per month).
- Downtime directly costs your business users, revenue, and credibility.

Open-source alternatives often fall short:
- Frequently outdated or abandoned.
- Resource-intensive or poorly optimized.
- Complex and burdensome to configure and deploy.
- Ineffective against advanced, modern attack vectors.

### The Hidden API Vulnerability

Most engineers don't realize that even when an API properly rejects unauthorized requests, the server still burns significant CPU and memory resources authenticating each request before rejecting it. This creates a devastating vulnerability:

Even though API requests will be rejected with 401/403 errors, the server must still:
1. Process each connection
2. Parse headers/JSON payload
3. Run authentication logic
4. Generate and send a response

Below is a simple but devastating command to flood an API endpoint. This single line can generate enough traffic to bring down many servers:

```bash
ab -n 100000 -c 1000 -H "Authorization: Bearer INVALID_TOKEN" -p payload.json -T application/json https://api.example.com/endpoint/
```

⚠️ **WARNING: FOR EDUCATIONAL PURPOSES ONLY** ⚠️  
DO NOT run this against systems you don't own or have permission to test.  
Install Apache Benchmark if needed: `apt-get install apache2-utils`

This command sends 100,000 requests with 1,000 concurrent connections. Even a well-resourced server can struggle under this load from a single machine. Imagine this distributed across thousands of compromised devices in a botnet.

This attack is even more devastating when:

1. **Distributed across thousands of IPs** - making IP-based rate limiting ineffective
2. **Targeting authentication endpoints** - which typically require expensive DB lookups and password hashing
3. **Executed from a botnet** - amplifying the attack by orders of magnitude

Traditional solutions like API keys, rate limiting, or WAFs merely reduce the impact but don't eliminate it—the server still must process each request to determine its legitimacy. 

Even if you're using cloud providers like AWS, Google Cloud, or Azure with auto-scaling capabilities, you're still vulnerable in a different way: **your bill**. A sustained DDoS attack can trigger massive auto-scaling, resulting in thousands or tens of thousands of dollars in unexpected infrastructure costs before you can respond.

IronShield's approach is fundamentally different: it creates a computational barrier that prevents attackers high volumes of request from reaching the protected servers in the first place.

### Why IronShield?

IronShield modernizes and drastically improves upon the conceptual foundation set by solutions like [PoW-Shield](https://github.com/RuiSiang/PoW-Shield.git), leveraging Rust and WASM performance, memory safety, and concurrency to provide sophisticated protection that's faster, safer, and dramatically more efficient.

Additionally, IronShield aims to be the "Vercel for Cloudflare®"—providing a user-friendly, no-code, 1-click security solution. While Cloudflare® offers powerful tools for technical users, IronShield bridges the gap, delivering streamlined, accessible cybersecurity to everyone.

Unlike previous solutions like PoW-Shield, which required running directly on your server and only filtering traffic after it had already reached your infrastructure, IronShield is specifically designed to operate on the edge. This means malicious traffic is intercepted and blocked before it ever touches your backend, greatly reducing the risk and resource drain on your systems.

> **Note:** PoW-Shield's need to run on your own server isn't even its biggest problem anymore. It's now unmaintained since [its really obvious drug kingpin maintainer was arrested for being a darknet drug kingpin](https://www.ice.gov/news/releases/incognito-market-owner-arrested-operating-one-largest-online-narcotics-marketplaces). In contrast, we can promise that our developers stick to coding, not running $100M dark web markets! Ironically, necessity drove innovation when darknet operators needed protection without relying on mainstream security providers (e.g. Cloudflare) using black-box bot detection and blocking that would just block every single Tor/VPN user. This inadvertently pioneered techniques that would later benefit legitimate privacy-focused services requiring bot protection without massive third-party dependencies.

### WebAssembly Edge Advantage

IronShield's innovative WebAssembly-powered security delivers unprecedented advantages:

- **15x Performance Improvement:** Our Rust-to-WebAssembly compilation dramatically outperforms JavaScript-based alternatives, completing proof-of-work challenges up to 15x faster.
- **Consistent Cross-Browser Execution:** Unlike JavaScript implementations with variable performance across browsers, our WebAssembly solution ensures uniform security enforcement across all platforms.
- **Precise Challenge Calibration:** The predictable, consistent performance allows for fine-tuned difficulty settings that work reliably across different devices and browsers.
- **Browser-Agnostic Security:** Works consistently across all modern browsers, headless environments, and automation tools without degradation.
- **Reduced User Friction:** Lightning-fast challenge completion (typically under 0.3 seconds) creates a seamless experience for legitimate users.
- **Predictable Resource Usage:** Our garbage-collector-free Rust implementation maintains consistent, low memory overhead with minimal performance variability.
- **Edge-Optimized Code:** Purpose-built for deployment on Cloudflare® Workers and other edge computing platforms.
- **Privacy Compatibility:** Works reliably with privacy tools like Tor and VPNs without compromising security.

### Key Features

- **Proof of Work (PoW) Challenges:** Forces attackers to consume significant CPU resources, making automated attacks economically unsustainable.
- **High-Speed Rust Proxy:** Built for extreme throughput and minimal latency, ensuring protection doesn't degrade user experience.
- **Adaptive Rate Limiting:** Intelligently allows legitimate traffic patterns while throttling malicious requests.
- **Advanced Web Application Firewall (WAF):** Detects and blocks malicious patterns before they reach your application.
- **SEO Bot Whitelisting & Verification:** Preserves search engine ranking by whitelisting known bots (e.g., Googlebot, Bingbot) and verifying their identity through reverse/forward DNS lookups to prevent User-Agent spoofing.
- **Distributed & Scalable:** Deploy horizontally across multiple nodes and infrastructure seamlessly.
- **Data Privacy:** Traffic stays securely within your infrastructure without routing through third-party services.
- **Minimal Resource Footprint:** Uses significantly fewer resources than Node.js or Python-based solutions.
- **AI Scraper Maze (Optional "AI Revenge Mode"):** Deploy decoy content designed specifically to confuse, poison, and frustrate AI-powered web scrapers and data crawlers.

### The Rust Advantage

IronShield's use of Rust delivers distinct benefits over JavaScript or Python-based security tools:

- **No Garbage Collection Pauses:** During critical security verification, JavaScript's unpredictable garbage collection can introduce timing vulnerabilities. Rust's deterministic memory management eliminates these pauses entirely.
- **Predictable Security Performance:** Cryptographic verification times remain consistent regardless of system load, preventing timing-based exploits.
- **Memory Safety Without Overhead:** Rust's ownership model prevents memory vulnerabilities without the performance penalties of managed runtimes.
- **WebAssembly Integration:** Rust compiles seamlessly to WebAssembly, enabling identical cryptographic logic on both server and client.
- **Concurrent Processing:** Efficient parallel hash computations without thread synchronization overhead.


### Ideal for:

- **Crypto Exchanges & DeFi Platforms:** Protect sensitive APIs from targeted attacks without blocking privacy-focused traffic.
- **Web3 Infrastructure Providers:** Guard critical API infrastructure with reliable security.
- **NFT Projects:** Ensure stability during high-demand mint events.
- **Blockchain Explorers:** Maintain uptime and data integrity under heavy load.
- **Privacy-First Services:** Avoid reliance on third-party security services and maintain compatibility with Tor and VPN users.
- **Self-Hosted Businesses:** Achieve enterprise-grade security on your own infrastructure.
- **Small and Medium Businesses:** Gain access to powerful security without breaking the bank.
- **Teams Demanding Full Stack Control:** Deploy and manage security independently, free from external dependencies.

### How IronShield Works

IronShield operates as a secure reverse proxy in front of your application:

1. **Client Request:** User requests access your application.
2. **PoW Challenge:** IronShield issues a computational challenge executed via JavaScript or WebAssembly.
3. **Verification:** The user's browser completes the computational puzzle, demonstrating genuine intent.
4. **Token Generation:** Verified clients receive a time-limited access token.
5. **Access Granted:** Requests with valid tokens seamlessly reach your application.
6. **Malicious Requests Blocked:** Bots and attackers, unable to afford the computational overhead, are effectively blocked before hitting your server.

For legitimate users, the computational cost is imperceptible. However, for attackers, it imposes a severe penalty, making large-scale attacks financially and computationally impractical.

### Future-Proof Your Security

IronShield plans to support self-hosted Web Application Firewall (WAF) deployments, offering complete flexibility for organizations requiring additional control or regulatory compliance. 

### Getting Started

Deploy IronShield effortlessly via Cloudflare® Workers or your own infrastructure:
- Clone the IronShield repository.
- Configure your security rules and PoW settings.
- Deploy using Docker, Kubernetes, or directly via Cloudflare® Workers.

### Join the Revolution in Edge Security

Stop compromising between security and cost-efficiency. Protect your platform with IronShield—enterprise-grade L7 DDoS protection designed specifically for your needs.

---

**IronShield: Built for Security. Optimized for Performance. Designed for Everyone.**
