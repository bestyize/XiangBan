# Build context: repository root (Nginx config lives in infra/deploy/nginx).
FROM node:24-alpine AS build
WORKDIR /web
# Override for environments where the default registry is slow or unreachable (e.g. mainland China previews).
ARG NPM_REGISTRY=https://registry.npmjs.org
COPY client/xiaban-web/package.json client/xiaban-web/package-lock.json ./
RUN npm config set registry "$NPM_REGISTRY" && npm ci --no-audit --no-fund
COPY client/xiaban-web ./
ARG VITE_PUBLIC_URL=https://xiaban.xyz
# Phase one runs outside mainland China and needs the separate consent; set to false after moving back.
ARG VITE_DATA_OVERSEAS=true
ENV VITE_PUBLIC_URL=$VITE_PUBLIC_URL VITE_DATA_OVERSEAS=$VITE_DATA_OVERSEAS
RUN npm run build

FROM nginx:1.28-alpine
COPY infra/deploy/nginx/nginx.conf /etc/nginx/nginx.conf
COPY infra/deploy/nginx/security-headers.conf /etc/nginx/security-headers.conf
COPY infra/deploy/nginx/busy.html /usr/share/nginx/html/busy.html
COPY --from=build /web/dist /usr/share/nginx/html
EXPOSE 80
