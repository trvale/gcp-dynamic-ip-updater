docker build -t gcp-dynamic-ip-updater .
echo "Copying docker image to containerd"
docker save gcp-dynamic-ip-updater:latest | k3s ctr images import -