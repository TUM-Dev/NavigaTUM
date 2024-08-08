# https://docs.docker.com/build/attestations/sbom/#scan-build-context
ARG BUILDKIT_SBOM_SCAN_CONTEXT=true

FROM node:21-alpine3.19 AS build-stage
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
ENV NITRO_SHUTDOWN=true
ENV NITRO_PRESET=node-cluster
RUN pnpm run build


FROM node:21-alpine3.19 AS production-stage
RUN apk update --no-progress --quiet && apk add curl --no-progress --quiet

COPY --from=build-stage /app/.output /app/.output
COPY --from=build-stage /app/node_modules /app/node_modules
COPY --from=build-stage /app/.nuxt /app/.nuxt

EXPOSE 3000
ENV NITRO_PORT=3000
ENV NITRO_HOST=0.0.0.0
ENV NITRO_SHUTDOWN=true
ENV NITRO_PRESET=node-cluster

HEALTHCHECK CMD curl --fail localhost:3000/health || exit 1
CMD node /app/.output/server/index.mjs
