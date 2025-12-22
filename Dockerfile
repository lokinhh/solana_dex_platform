# syntax=docker/dockerfile:1

FROM node:22-alpine AS api-deps
WORKDIR /app/backend
COPY backend/package*.json ./
RUN npm ci --omit=dev

FROM node:22-alpine AS api
WORKDIR /app
RUN apk add --no-cache tini wget
COPY --from=api-deps /app/backend/node_modules ./backend/node_modules
COPY backend ./backend
ENV NODE_ENV=production PORT=8091
EXPOSE 8091
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["node", "backend/src/server.js"]

FROM node:22-alpine AS web-deps
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci

FROM node:22-alpine AS web-builder
WORKDIR /app/frontend
COPY --from=web-deps /app/frontend/node_modules ./node_modules
COPY frontend ./
ENV NEXT_TELEMETRY_DISABLED=1
RUN npm run build

FROM node:22-alpine AS web
WORKDIR /app/frontend
RUN apk add --no-cache tini
COPY --from=web-builder /app/frontend/.next ./.next
COPY --from=web-builder /app/frontend/node_modules ./node_modules
COPY --from=web-builder /app/frontend/package.json ./package.json
COPY --from=web-builder /app/frontend/public ./public
ENV NODE_ENV=production PORT=3000
EXPOSE 3000
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["npm", "start"]
