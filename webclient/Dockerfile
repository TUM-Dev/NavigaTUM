# https://docs.docker.com/build/attestations/sbom/#scan-build-context
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true

FROM node:24-alpine3.21 AS base

RUN apk update --no-progress --quiet && \
    apk add --no-cache --no-progress --quiet curl sqlite sqlite-libs

ENV NODE_ENV=production
ENV NITRO_PRESET=node-server
ENV NITRO_PORT=3000
ENV NITRO_HOST=0.0.0.0

ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
ARG GIT_COMMIT_MESSAGE
ENV GIT_COMMIT_MESSAGE=${GIT_COMMIT_MESSAGE}

WORKDIR /app

FROM base AS build-stage

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY package.json ./
COPY pnpm-lock.yaml ./
COPY tsconfig.json .
COPY ./*.ts .
RUN pnpm install --frozen-lockfile

COPY --link public public
COPY --link content content
COPY --link app app
RUN pnpm run postinstall
RUN pnpm run build && test -f /app/.output/server/index.mjs

FROM base AS production-stage

COPY --from=build-stage /app/.output /app/.output
COPY --from=build-stage /app/node_modules /app/node_modules
COPY --from=build-stage /app/.nuxt /app/.nuxt

EXPOSE 3000

HEALTHCHECK CMD curl --fail localhost:3000/health || exit 1
CMD ["node", "/app/.output/server/index.mjs"]
