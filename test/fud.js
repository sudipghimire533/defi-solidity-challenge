const { expect } = require("chai");
const { mine } = require("@nomicfoundation/hardhat-network-helpers");

// Ways to improve test:
// - split each test into it's own new chai test
// - test for all verbose behaviour including: traver fails when amount > what address have

describe("FUD contract basics", function () {
  it("Basics ERC-20 functionality should be satisfied", async function () {
    const [owner, alice, bob] = await ethers.getSigners();

    const FudToken = await ethers.deployContract("FudToken");

    // Test constants
    expect(await FudToken.totalSupply()).to.equal(1500000);
    expect(await FudToken.symbol()).to.equal("FUD");
    expect(await FudToken.name()).to.equal("FUD Token");

    // Test transfer
    // transfer 1000 to alice
    await expect(FudToken
      .connect(owner)
      .transfer(alice.address, 1000))
      .to.emit(FudToken, "Transfer").withArgs(owner.address, alice.address, 1000);
    expect(await FudToken.balanceOf(owner.address)).to.equal(1500000 - 1000);
    expect(await FudToken.balanceOf(alice.address)).to.equal(1000);

    // Test approve
    await expect(FudToken
      .connect(owner)
      .approve(bob.address, 1000))
      .to.emit(FudToken, "Approval").withArgs(owner.address, bob.address, 1000);
    expect(await FudToken.allowance(owner.address, bob.address)).to.equal(1000);

    // Test TransferFrom
    await expect(FudToken
      .connect(bob)
      .transferFrom(owner.address, alice.address, 500))
      .to.emit(FudToken, "Transfer").withArgs(owner.address, alice.address, 500);
    expect(await FudToken.balanceOf(alice.address)).to.equal(1500);
  });
});

describe("FUD contract basics", function () {
  it("Minting functionality should be ok", async function () {
    const [owner, alice, bob] = await ethers.getSigners();

    const winTokenFactory = await ethers.getContractFactory("WinToken");
    const WinToken = await winTokenFactory.deploy(owner); // owner is the minter

    // mint and verify
    await expect(WinToken.connect(owner).mint(alice, 1000))
      .to.emit(WinToken, "Transfer").withArgs(ethers.ZeroAddress, alice.address, 1000);
    expect(await WinToken.balanceOf(alice)).to.equal(1000);

    // make sure others than minter cannot mint and is revrted with "unauthorised access"
    await expect(WinToken.connect(bob).mint(alice, 1000)).to.be.revertedWith("Unauthorised mint");

  });
});

describe("AirVault contract basics", function () {
  it("AirVault basic functionality should be ok", async function () {
    const [owner, alice, bob] = await ethers.getSigners();

    const FudToken = await ethers.deployContract("FudToken");
    const AirVaultFactory = await ethers.getContractFactory("AirVault");
    const AirVault = await AirVaultFactory.deploy(FudToken.target);

    // transfer some FUD to alice
    await FudToken
      .connect(owner)
      .transfer(alice.address, 1000);

    // alice will deposit some fud to airvault
    // before he have to allow airvault to spend on his behalf
    await FudToken.connect(alice).approve(AirVault.target, 200);

    await mine(10);
    let depositBlock = await ethers.provider.getBlockNumber();
    
    await expect(AirVault.connect(alice).deposit(200))
      .to.emit(AirVault, "Deposited").withArgs(alice.address, 200, depositBlock + 1);
    expect(await FudToken.balanceOf(alice)).to.equal(800);
    expect(await AirVault.lockedBalanceOf(alice)).to.equal(200);

    await mine(5);
    let withdrawBlock = await ethers.provider.getBlockNumber();

    // allow withdraw
    await expect(AirVault.connect(alice).withdraw(100))
      .to.emit(AirVault, "Withdrawn").withArgs(alice.address, 100, withdrawBlock + 1);
    expect(await FudToken.balanceOf(alice)).to.equal(900);
    expect(await AirVault.lockedBalanceOf(alice)).to.equal(100);
  });
});
