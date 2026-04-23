FROM node:20-bullseye-slim AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
# Fetch Linux native rollup binary (lockfile generated on Windows)
RUN npm install @rollup/rollup-linux-x64-gnu@4.60.1 --no-save
RUN npm run build

FROM alpine:3.21
RUN apk add --no-cache nginx && mkdir -p /run/nginx
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/http.d/default.conf
EXPOSE 3000
CMD ["nginx", "-g", "daemon off;"]
