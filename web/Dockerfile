# Build context: repository root (Nginx config lives in deploy/nginx).
FROM node:24-alpine AS build
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web ./
ARG VITE_PUBLIC_URL=https://xiaban.xyz
# Phase one runs outside mainland China and needs the separate consent; set to false after moving back.
ARG VITE_DATA_OVERSEAS=true
ENV VITE_PUBLIC_URL=$VITE_PUBLIC_URL VITE_DATA_OVERSEAS=$VITE_DATA_OVERSEAS
RUN npm run build

FROM nginx:1.28-alpine
COPY deploy/nginx/nginx.conf /etc/nginx/nginx.conf
COPY deploy/nginx/security-headers.conf /etc/nginx/security-headers.conf
COPY deploy/nginx/busy.html /usr/share/nginx/html/busy.html
COPY --from=build /web/dist /usr/share/nginx/html
EXPOSE 80
