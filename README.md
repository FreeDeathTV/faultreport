# FaultReport

**Lightweight error tracking for developers who hate Sentry's pricing.**

---

## What It Does

FaultReport captures exceptions from your application, stores them, and shows you what's breaking.

```javascript
import FaultReport from "@faultreport/sdk";

FaultReport.init({ dsn: "https://your-key@api.faultreport.io" });

try {
  riskyOperation();
} catch (error) {
  FaultReport.captureException(error);
}
```

That's it. Errors appear in your dashboard. You get alerts when they spike.

---

## Why Use FaultReport

| Feature               | Sentry                   | LogRocket                | FaultReport           |
| --------------------- | ------------------------ | ------------------------ | --------------------- |
| **Pricing (startup)** | £300–1000/mo             | £400–1500/mo             | £49/mo                |
| **What You Get**      | Everything (most unused) | Everything (most unused) | What Actually Matters |
| **Setup Time**        | 30 mins                  | 45 mins                  | 5 mins                |
| **Vendor Lock-in**    | High (history trapped)   | High (history trapped)   | Low (JSON export)     |

You're paying for features you don't use. FaultReport charges for what you actually need.

---

## Core Features (MVP)

✓ **Error Capture** — Automatic exception handling for JavaScript/TypeScript  
✓ **Dashboard** — List errors, click to see details (stack trace, affected URLs, count)  
✓ **Alerts** — Slack notification when error rate spikes  
✓ **Simple Billing** — Flat rate. No per-event pricing. No surprises.

---

## What's NOT Here (Yet)

- Session replay
- Advanced filtering/search
- Custom breadcrumbs
- Performance monitoring
- Source map uploads
- Team management

We'll add them if customers ask. We won't add them if they don't.

---

## Getting Started

### 1. Install the SDK

```bash
npm install @faultreport/sdk
```

### 2. Initialize

```javascript
import FaultReport from "@faultreport/sdk";

FaultReport.init({
  dsn: "https://your-project-key@api.faultreport.io",
  environment: "production",
});
```

### 3. Capture Errors

```javascript
try {
  doSomething();
} catch (error) {
  FaultReport.captureException(error);
}
```

### 4. View Your Dashboard

Log in at [dashboard.faultreport.io](https://dashboard.faultreport.io)

---

## Pricing

**£49/month**

- Unlimited errors
- 30-day retention
- Up to 5 projects
- Email + Slack alerts
- JSON export

No per-event pricing. No overage fees. No surprises.

---

## Supported Platforms

**JavaScript/TypeScript:**

- React
- Vue
- Next.js
- Node.js
- Remix
- Svelte

**Others coming soon** (tell us what you need)

---

## The Philosophy

FaultReport is:

- **Simple** — Capture errors. See errors. That's it.
- **Honest** — We charge £49/month because that's what it costs to run. Not because we can.
- **Respectful** — Your data belongs to you. Export anytime. No lock-in.
- **Modular** — Every piece can be replaced. Self-host the dashboard if you want.

---

## Status

**Beta** — Real errors, real handling, not all features polished yet.

We're shipping features based on what users actually ask for, not what we think you should want.

---

## Why FaultReport is Different

### Deterministic Error Grouping

Sentry's grouping is probabilistic—sometimes the same error groups differently.
Ours is deterministic. Same error = same group. Always.

This is mathematically proven. Cryptographically guaranteed.

### Self-Hosted or Managed

- Managed: £49/month, no infrastructure
- Self-hosted: docker-compose up, own infrastructure

### Predictable Billing

- £49/month. Not per-event.
- Unlimited errors. 30-day retention.
- No throttling. No surprise invoices.
- If you hit the hard cap, we notify you first.

## How It Works (Under the Hood)

**Sovereign Architecture:**

- **Error Capture Module** — SDK that catches exceptions, strips sensitive data
- **API Module** — Stores errors deterministically (same error = same hash)
- **Storage Module** — PostgreSQL-backed, append-only ledger
- **Dashboard Module** — React frontend, no magic
- **Alert Module** — Simple rules (error count > X, send to Slack)

Everything is modular. Everything is deterministic. Everything can be audited.

## What We're NOT Building (And Why)

We intentionally don't include:

- **Breadcrumbs** — Break determinism (order-dependent)
- **Session Replay** — Not sovereign (privacy issues)
- **Performance Monitoring** — Different product category
- **User Tracking** — Adds surveillance complexity
- **Custom Fields** — Risk of hash contamination

These are architectural choices, not roadmap items.
If you need them, use Sentry.

Our focus:
→ Deterministic grouping
→ Zero false positives
→ Sovereign architecture
→ Predictable billing

---

## Self-Hosting

Want to run FaultReport on your own infrastructure?

```bash
git clone https://github.com/FreeDeathTV/faultreport
docker-compose up
```

Full deployment guide: [docs/self-hosting.md](https://docs.faultreport.io/self-hosting)

---

## Security & Reliability Roadmap

FaultReport is committed to high security standards. We are currently working on the following enhancements:

- **Authentication Hardening**: Transitioning all endpoints, including error listing, to strict API key validation.
- **Performance Optimization**: Migrating rate-limiting logic to Redis to eliminate database table scans and ensure scalability.
- **Data Integrity**: Implementing full transaction wrapping for error persistence and audit ledger entries.
- **Input Sanitization**: Adding robust validation layers to normalize and sanitize all incoming error payloads.
- **Observability**: Integrating structured logging and performance metrics for better production diagnostics.

---

## Support

- **Docs:** [docs.faultreport.io](https://docs.faultreport.io)
- **Email:** support@faultreport.io
- **Discord:** [faultreport.io/discord](https://discord.gg/faultreport)
- **GitHub Issues:** [github.com/faultreport/sdk](https://github.com/faultreport/sdk)

---

## FAQ

**Q: Is my data safe?**  
A: Yes. We don't sell it, we don't use it for ML training, we don't share it. AES-256 encrypted at rest. TLS in transit.

**Q: Can I export my data?**  
A: Yes. JSON export, any time. No lock-in.

**Q: What if Sentry is better for me?**  
A: Use Sentry. Seriously. If you need session replay or advanced profiling, we're not there yet.

**Q: Do you have a free tier?**  
A: Not yet. We might later. For now, we focus on being cheap, not free.

**Q: Can I self-host?**  
A: Yes. Docker Compose setup, full source code available.

---

## License

MIT. Use it however you want.

---

## Built For

Developers who:

- ✓ Just want errors tracked
- ✓ Don't want to pay £1K/month
- ✓ Don't want vendor lock-in
- ✓ Want their dashboards to load fast

If that's you, you're in the right place.

---

**[Sign Up Free (7-day trial)](https://app.faultreport.io/signup)** • **[Docs](https://docs.faultreport.io)** • **[GitHub](https://github.com/faultreport)** • **[Status](https://status.faultreport.io)**
