type Point3D = [number, number, number];

type Image3D = {
  PointCloudImage: Point3D[];
};

type SendableData = Image3D;

function handleWebSocketMessage(event: MessageEvent) {
  try {
    const parsedData: SendableData = JSON.parse(event.data);

    console.log("Received PointCloudImage data:", parsedData.PointCloudImage);
  } catch (error) {
    console.error("Failed to parse WebSocket message:", error);
  }
}

const ws = new WebSocket("/");

ws.onmessage = handleWebSocketMessage;

ws.onopen = () => {
  console.log("WebSocket connection opened.");
};

ws.onerror = (error) => {
  console.error("WebSocket error:", error);
};

ws.onclose = () => {
  console.log("WebSocket connection closed.");
};
