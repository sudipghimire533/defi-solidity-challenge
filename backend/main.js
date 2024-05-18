const ethers = require('ethers');

// Replace with your provider URL (e.g., Infura, Alchemy)
const provider = new ethers.providers.JsonRpcProvider('127.0.0.1:8545');

// Contract details (replace with actual address and event signature)
const contractAddress = '0x5FbDB2315678afecb367f032d93F642f64180aa3';
const transferEventSignature = 'Transfer(address,address,uint256)';

async function listenForTransfers() {
  try {
    const filter = {
      address: contractAddress,
      topics: [ethers.utils.id(transferEventSignature)],
    };

    const contract = new ethers.Contract(contractAddress, provider); // Optional: Use contract ABI if needed

    const subscription = await provider.on(filter, async (from, to, value, event) => {
      console.log(`Transfer detected!`);
      console.log(`  From: ${from}`);
      console.log(`  To: ${to}`);
      console.log(`  Value: ${ethers.utils.formatUnits(value, 18)} ETH`); // Format for readability

      // Add your custom logic here to process the event data (e.g., database updates, notifications)
      // Example: Send a notification
      // await sendNotification({ from, to, value });
    });

    console.log(`Listening for Transfer events on contract ${contractAddress}`);

    // Handle uncaught exceptions within the subscription callback
    subscription.on('error', (error) => {
      console.error('Error during Transfer event processing:', error);
    });

  } catch (error) {
    console.error('Error initializing listener:', error);
  }
}

// Optional: Function to send a notification (replace with your implementation)
async function sendNotification(data) {
  // Implement your notification logic here (e.g., email, SMS, logging)
  console.log('Sending notification:', data);
}

listenForTransfers();
