const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

const { FUD_TOKEN } = process.env;

module.exports = buildModule("AirVault", (m) => {
  let fudTokenAddress = ( FUD_TOKEN == null)? "0x5FbDB2315678afecb367f032d93F642f64180aa3" : FUD_TOKEN;

  const lock = m.contract("AirVault", [fudTokenAddress]);

  return { lock };
});
