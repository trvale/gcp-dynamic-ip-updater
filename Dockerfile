FROM ghcr.io/astral-sh/uv:alpine3.20

# copy the script into the container
COPY gcp-dynamic-ip-updater.py /gcp-dynamic-ip-updater.py
RUN apk add --no-cache python3~=3.12

# run the script
CMD ["uv", "run", "gcp-dynamic-ip-updater.py", "unifi-controller", "ssh-from-home-network"]