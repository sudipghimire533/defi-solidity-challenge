const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("WinToken", (m) => {
  const lock = m.contract("FudToken");

  return { lock };
});
