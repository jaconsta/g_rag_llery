// Basic service worker for SvelteKit PWA
self.addEventListener('install', () => { // event => {
  self.skipWaiting();
});

self.addEventListener('activate', () => {
  self.clients.claim();
});

// Listen for messages from the main thread for uploads
self.addEventListener('message', async event => {
  console.log("heyy");
  if (!self.uploadCancelFlag) self.uploadCancelFlag = false;
  if (event.data && event.data.type === 'CANCEL_UPLOAD') {
    self.uploadCancelFlag = true;
    return;
  }
  if (event.data && event.data.type === 'UPLOAD_IMAGES') {
    self.uploadCancelFlag = false;
    const files = event.data.files;
    let cancelledAt = -1;
    for (let i = 0; i < files.length; i++) {
      const fileData = files[i];
      if (self.uploadCancelFlag) {
        cancelledAt = i;
        break;
      }
      // Upload data
      console.log("pushh")
      const formData = new FormData();
      formData.append("file", fileData.buffer, "filename");
      formData.append("name", fileData.name);
      console.log("pushh send")

      const response = await fetch("/api/uploads", {
        method: "POST",
        headers: {
          "Content-Type": "multipart/form-data"
        },
        body: formData
      });

      const responseBody = await response.json(); // : { success: bool }
      const status = responseBody.success ? "finished" : "error";
      console.log("did somthing");

      // Simulate upload delay
      // await new Promise(r => setTimeout(r, 1200 + Math.random() * 1200));
      if (self.uploadCancelFlag) {
        cancelledAt = i + 1;
        break;
      }
      // Notify client of upload result
      const allClients = await self.clients.matchAll();
      for (const client of allClients) {
        client.postMessage({
          type: 'UPLOAD_STATUS',
          name: fileData.name,
          size: fileData.size,
          status,
        });
      }
    }
    // If cancelled, send 'stopped' status for remaining files
    if (cancelledAt !== -1) {
      for (let j = cancelledAt; j < files.length; j++) {
        const fileData = files[j];
        const allClients = await self.clients.matchAll();
        for (const client of allClients) {
          client.postMessage({
            type: 'UPLOAD_STATUS',
            name: fileData.name,
            size: fileData.size,
            status: 'stopped',
          });
        }
      }
    }
  }
});

// Background Sync for uploads
self.addEventListener('sync', event => {
  if (event.tag === 'upload-images') {
    // Could trigger upload logic here if needed
  }
});
