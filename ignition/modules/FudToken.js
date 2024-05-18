const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("FudToken", (m) => {
  const lock = m.contract("FudToken");

  return { lock };
});
