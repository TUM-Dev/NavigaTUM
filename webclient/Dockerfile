FROM node:21 as build-stage
WORKDIR /app
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY package.json ./
COPY pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY . .
ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
ARG GIT_COMMIT_MESSAGE
ENV GIT_COMMIT_MESSAGE=${GIT_COMMIT_MESSAGE}
RUN npm run build-only
RUN gzip --force --keep --recursive ./dist


FROM nginx:1.25 as production-stage
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir /app && apt update && apt upgrade -y
COPY --from=build-stage /app/dist /app

EXPOSE 3000
HEALTHCHECK CMD curl --fail localhost:3000/health || exit 1
CMD sh -c "sed -i 's|TILESERVER_URL|${TILESERVER_URL}|g' /etc/nginx/nginx.conf && sed -i 's|CDN_URL|${CDN_URL}|g' /etc/nginx/nginx.conf && sed -i 's|MAIN_API_URL|${MAIN_API_URL}|g' /etc/nginx/nginx.conf && sed -i 's|FEEDBACK_API_URL|${FEEDBACK_API_URL}|g' /etc/nginx/nginx.conf && nginx -g 'daemon off;'"
