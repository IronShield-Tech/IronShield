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

### Key Features

- **Proof of Work (PoW) Challenges:** Forces attackers to consume significant CPU resources, making automated attacks economically unsustainable.
- **High-Speed Rust Proxy:** Built for extreme throughput and minimal latency, ensuring protection doesn't degrade user experience.
- **Adaptive Rate Limiting:** Intelligently allows legitimate traffic patterns while throttling malicious requests.
- **Advanced Web Application Firewall (WAF):** Detects and blocks malicious patterns before they reach your application.
- **SEO Bot Whitelisting & Verification:** Preserves search engine ranking by whitelisting known bots (e.g., Googlebot, Bingbot) and verifying their identity through reverse/forward DNS lookups to prevent User-Agent spoofing.
- **Distributed & Scalable:** Deploy horizontally across multiple nodes and infrastructure seamlessly.
- **Privacy Compatibility:** Works reliably with privacy tools like Tor and VPNs without compromising security. Traditional black box heuristic bot detection and prevention implementations outright block these users simply for using a VPN or Tor. Ironshield allows legitimate users using these privacy tools a way in without compromising on security.
- **Minimal Resource Footprint:** Uses significantly fewer resources than Node.js or Python-based solutions.
- **AI Scraper Maze (Optional "AI Revenge Mode"):** Deploy decoy content designed specifically to confuse, poison, and frustrate AI-powered web scrapers and data crawlers.

### WebAssembly Edge Advantage

IronShield's innovative WebAssembly-powered security delivers unprecedented advantages:

- **15x Performance Improvement:** Our Rust-to-WebAssembly compilation dramatically outperforms JavaScript-based alternatives, completing proof-of-work challenges up to 15x faster. For example, to solve a challenge with base difficulty 5, an optimized multithreaded javascript solution could compute a hash with 5 leading zeroes on an m1 macbook in around 10-15 seconds. The Rust WASM version allows it to do the same computation in around 350ms.
- **Consistent Cross-Browser Execution & Calibration:** Unlike JavaScript implementations with variable performance across browsers, our WebAssembly solution ensures uniform security enforcement across all platforms. For example, Chromium based browsers for some reason can hash 25% faster than Firefox based browsers when utilizing traditional js hashing methods (crypto.subtle.digest). The predictable, consistent performance also allows for fine-tuned difficulty settings that work reliably across different devices, browsers, headless environments, and automation tools.
- **Reduced User Friction:** Lightning-fast challenge completion (typically under 0.3 seconds) creates a seamless experience for legitimate users.
- **Edge-Optimized Code:** Purpose-built for deployment on Cloudflare® Workers and other edge computing platforms.

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

### The Economics of DDoS Protection

To illustrate how IronShield creates an economic barrier against attacks, let's analyze a real-world scenario:

**Scenario: Large-Scale DDoS Attack**
- Attack volume: 100,000 requests per second (RPS)
- PoW challenge: Each request requires 0.3 seconds of CPU time (Ironshield's black box bot detection would likely serve likly bots challenges that are harder and take 3-10s, but we'll assume a worst-case scenario where they spoof a legitimate browser perfectly)
- Target: A service protected by IronShield

**Computational Requirements for the Attacker:**
- 100,000 RPS × 0.3 CPU-seconds = 30,000 CPU-core-seconds needed per second
- This translates to 30,000 CPU cores running continuously

**Financial Impact at Current AWS EC2 Pricing:**
- Average compute-optimized instance cost: ~$0.042 per vCPU hour (AWS prices)
- 30,000 CPU cores × $0.042 = $1,260 per hour
- $1,260 × 24 hours = $30,240 per day

Without IronShield, this attack would likely overwhelm your infrastructure, causing downtime or triggering massive auto-scaling costs. With IronShield, the economics are reversed - the attacker must spend over huge sums of money daily to maintain the attack, while your protected infrastructure handles legitimate traffic efficiently.

**Advanced Scenario: Enhanced Bot Protection**
For suspected bots and attackers, IronShield can dynamically increase challenge difficulty:
- If challenges are increased to 11 seconds of CPU time per request:
  - 100,000 RPS × 11 CPU-seconds = 1,100,000 CPU-core-seconds needed per second
  - This requires 1,100,000 CPU cores running continuously
  - 1,100,000 CPU cores × $0.042 = $46,200 per hour
  - $46,200 × 24 hours = $1,108,800 per day (~$1.1 million daily)

This demonstrates why PoW-based protection is so effective: it transforms a DDoS attack from a technical challenge into an economic one, where attackers must continuously pay substantially more than their targets. Meanwhile, legitimate users experience minimal latency (less than 0.3 seconds) for challenge completion.

### The Reality of Botnet Threats

While theoretical concerns about botnets using specialized hardware like GPU's and ASIC's to "game" the PoW hashing calculations might seem worrying, the practical reality of DDoS attacks is quite different:

**Common Botnet Composition:**
- Most botnets consist of compromised consumer devices (laptops, desktops, IoT devices)
- Enterprise cloud instances (like hijacked AWS compute)
- Standard servers with conventional CPUs

**Why Specialized Hardware Botnets Are Uncommon:**
1. **Economic Misalignment:** Attackers with access to GPU farms or ASICs have far more profitable opportunities in cryptocurrency mining - using this hardware for DDoS represents a significant opportunity cost
2. **Technical Barriers:** Utilizing victim GPUs requires specialized malware capabilities beyond most botnet operators
3. **Operational Complexity:** Managing GPU resources remotely adds detection risk and development overhead

Typical botnets lack the specialized hardware or software engineering capabilities to efficiently bypass the computational barriers. Standard CPU-based botnets face the full economic cost of the PoW system. Even with extremely challenging PoW difficulty settings that a specialized ASIC/GPU attack could theorectically bypass, the aforementioned barriers would be quite good at preventing those.

**Multi-Layered Defense:**
IronShield doesn't rely solely on computational challenges. Our heuristic bot classification system:
- Analyzes browser fingerprints and behavior patterns
- Dynamically adjusts challenge difficulty based on suspicion level
- Provides similar protections to Cloudflare's bot detection but without blocking legitimate privacy-focused users

This multi-layered approach ensures that even in the unlikely event of specialized hardware being deployed, the combined defenses remain effective while maintaining compatibility with privacy tools like Tor and VPNs.

### Getting Started

Deploy IronShield effortlessly via Cloudflare® Workers or your own infrastructure:
- Clone the IronShield repository.
- Configure your security rules and PoW settings.
- Deploy using Docker, Kubernetes, or directly via Cloudflare® Workers.

## Project Structure

The project follows a modular structure:

- `ironshield-core` - Core library containing shared functionality
- `ironshield-wasm` - WebAssembly bindings for browser-side code
- `ironshield-cloudflare` - Cloudflare Worker implementation
- `assets` - Static assets for the project

## Setting Up and Running the Project

### Prerequisites

You'll need to install:

1. **Rust and Cargo** - [Install from rustup.rs](https://rustup.rs/)
2. **Node.js and npm** - [Install from nodejs.org](https://nodejs.org/)
3. **wasm-pack** - Install with:
   ```bash
   cargo install wasm-pack
   ```
4. **worker-build** - Install with:
   ```bash
   cargo install worker-build
   ```

### Installing the Nightly Toolchain for Threading Support

IronShield uses WebAssembly threading for optimal performance. You'll need to install the Rust nightly toolchain for your platform:

#### macOS (Intel or Apple Silicon)
```bash
# For Apple Silicon (M1/M2/M3)
rustup toolchain install nightly-aarch64-apple-darwin

# For Intel Macs
rustup toolchain install nightly-x86_64-apple-darwin
```

#### Linux
```bash
# For x86_64 architecture (most common)
rustup toolchain install nightly-x86_64-unknown-linux-gnu

# For ARM64 architecture
rustup toolchain install nightly-aarch64-unknown-linux-gnu
```

#### Windows
```bash
rustup toolchain install nightly-x86_64-pc-windows-msvc
```

You can verify the installation with:
```bash
rustup toolchain list
```

This enables the build script to use WebAssembly threading features (+atomics, +bulk-memory, +mutable-globals) which significantly improves performance for proof-of-work calculations.

### Installing Project Dependencies

Before running the project, install all required npm dependencies:

```bash
npm install
```

This will install all packages specified in package.json, including development dependencies like Wrangler that are necessary for building and running the project.

### Running the Project

A single command will build everything and start a local development server:

```bash
npx wrangler dev
```

This command:
1. Builds the WASM module
2. Compiles the Cloudflare Worker
3. Starts a local development server on http://localhost:8787

Alternatively, you can use the npm script:

```bash
npm run dev
```

### Building WASM Only

If you only want to build the WebAssembly module:

```bash
npm run build
```

### Deployment

To deploy to Cloudflare Workers:

```bash
npm run deploy
```
or
```bash
npx wrangler publish
```

## Development Workflow

1. Make changes to core functionality in `ironshield-core/src/`
2. Update WASM bindings in `ironshield-wasm/src/`
3. Modify Worker code in `ironshield-cloudflare/src/`
4. Run `npx wrangler dev` to test your changes

## Features

- Client-side proof-of-work in WebAssembly
- Efficient SHA-256 hashing
- Automatic difficulty adjustment
- Custom challenge page

## Troubleshooting

- If you see errors about missing assets, ensure you've run the build process at least once
- For WASM errors, check that wasm-pack is installed correctly
- For Worker errors, verify your Cloudflare credentials are configured

## License

[BSL License](LICENSE.BSL)
