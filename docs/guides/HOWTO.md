Here’s a **HOW‑TO‑USE FAULTREPORT** written so simply a caffeinated monkey could follow it.  
Short, blunt, zero thinking required.

---

# **FAULTREPORT — HOW TO USE IT**

## 1. **Install the Thing**

Open your terminal. Type this. Press Enter.

```bash
npm install @faultreport/sdk
```

If it errors, try again. If it still errors, Google “how to install Node”.

---

## 2. **Initialise the Thing**

Put this at the top of your app.

```javascript
import FaultReport from "@faultreport/sdk";

FaultReport.init({
  dsn: "https://your-project-key@api.faultreport.io",
  environment: "production",
});
```

If you don’t know your DSN, go to the dashboard and copy it.  
If you still can’t find it, shout at your nearest developer.

---

## 3. **Capture Errors**

Wrap your dangerous code in a try/catch.  
When it explodes, send it to FaultReport.

```javascript
try {
  doSomething();
} catch (error) {
  FaultReport.captureException(error);
}
```

That’s it. No magic. No rituals. No 40‑minute Sentry setup.

---

## 4. **Look at Your Errors**

Open your browser. Go here:

**[https://dashboard.faultreport.io](https://dashboard.faultreport.io)**

You’ll see:

- A list of errors
- How many times they happened
- Where they happened
- The stack trace
- A big number that goes up when things break

If nothing appears, break your app on purpose.  
(Every developer does this. It’s fine.)

---

## 5. **Get Alerts**

Connect Slack in the dashboard.

When errors spike, FaultReport shouts at you.  
Not every time. Only when it actually matters.

---

## 6. **Pay the £49**

If you like it, pay the £49/month.

- Unlimited errors
- 30‑day retention
- No surprise invoices
- No “you hit your quota, pay us £900” emails

If you don’t like it, export your JSON and leave.  
We won’t cry.

---

## 7. **Supported Stuff**

If your project uses:

- React
- Vue
- Next.js
- Node.js
- Remix
- Svelte

…it works.  
If you use something weird, tell us.

---

## 8. **Self‑Hosting (For the Brave)**

If you want to run it yourself:

```bash
git clone https://github.com/FreeDeathTV/faultreport
docker-compose up
```

If this doesn’t work, you probably shouldn’t self‑host.

---

## 9. **What FaultReport Does NOT Do**

So you don’t get confused:

❌ Session replay  
❌ Performance monitoring  
❌ Breadcrumbs  
❌ User tracking  
❌ Fancy dashboards

FaultReport is for **errors only**.  
If you want a surveillance tool, use Sentry.

---

## 10. **You’re Done**

You now have:

- Error tracking
- Deterministic grouping
- A dashboard that loads fast
- Billing that won’t ruin your month

Congratulations. You’ve successfully used FaultReport.
