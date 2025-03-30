docker build -t gcp-dynamic-ip-updater .
echo "Copying docker image to containerd"
docker save gcp-dynamic-ip-updater:latest -o /tmp/gcp-dynamic-ip-updater.tar
ctr image import /tmp/gcp-dynamic-ip-updater.tar
rm -f /tmp/gcp-dynamic-ip-updater.tar