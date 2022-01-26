FROM gcr.io/distroless/cc-debian10 as build-stage
WORKDIR /app
# collect data

# put all data into the cdn folder
RUN mkdir /build
COPY output/* /build/
COPY external/maps/* /build/maps/
COPY sources/img/* /build/img/

FROM nginx as production-stage
RUN mkdir /cdn
COPY --from=build-stage /build /cdn
COPY nginx.conf /etc/nginx/nginx.conf
