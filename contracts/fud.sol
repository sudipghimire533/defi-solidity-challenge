pragma solidity ^0.8.0;

contract ERC20 {
    mapping(address => uint256) public _balances;
    mapping(address => mapping(address => uint256)) public _allowances;

    uint256 private _totalSupply;
    string private _name;
    string private _symbol;

    // Events
    event Transfer(address indexed from, address indexed to, uint256 value);

    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
    }

    function name() public view returns (string memory) {
        return _name;
    }

    function symbol() public view returns (string memory) {
        return _symbol;
    }

    function decimals() public pure returns (uint8) {
        return 18;
    }

    function totalSupply() public view returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address account) public view returns (uint256) {
        return _balances[account];
    }

    function allowance(
        address owner,
        address spender
    ) public view returns (uint256) {
        return _allowances[owner][spender];
    }

    function _mint(address account, uint256 amount) internal virtual {
        require(account != address(0), "ERC20: mint to the zero address");

        _totalSupply += amount;
        _balances[account] += amount;
        emit Transfer(address(0), account, amount);
    }

    function transfer(
        address to,
        uint256 amount
    ) external returns (bool success) {
        address from = msg.sender;

        // Check pre-condition
        require(from != address(0), "Now allowed transfer from Zero address");
        require(to != address(0), "Now allowed transfer to Zero address");
        require(_balances[from] >= amount, "Not enough tokens");

        // do the transfer
        unchecked {
            _balances[msg.sender] -= amount;
        }
        _balances[to] += amount;

        // emit an event
        emit Transfer(msg.sender, to, amount);

        // return success ok
        return true;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        address owner = msg.sender;
        _increase_allowance(owner, spender, amount);
        emit Approval(owner, spender, amount);

        return true;
    }

    function transferFrom(
        address from,
        address to,
        uint256 amount
    ) external returns (bool) {
        address spender = msg.sender;

        require(_balances[from] >= amount, "Not enough tokens");

        _decrease_allowance(from, spender, amount);
        _balances[to] += amount;
        _balances[from] -= amount;

        emit Transfer(from, to, amount);
        return true;
    }

    function _increase_allowance(
        address owner,
        address spender,
        uint256 amount
    ) internal {
        _allowances[owner][spender] += amount;
    }

    function _decrease_allowance(
        address owner,
        address spender,
        uint256 amount
    ) internal {
        require(amount <= _allowances[owner][spender], "Not enough tokens");
        _allowances[owner][spender] -= amount;
    }
}

contract FudToken is ERC20 {
    mapping(address => uint256) _locks;

    constructor() ERC20("FUD Token", "FUD") {
        _mint(msg.sender, 1500000);
    }
}

contract WinToken is ERC20 {
    address _minter;

    constructor(address minter) ERC20("WIN Token", "WIN") {
        _minter = minter;
    }

    modifier OnlyMinter {
        require(msg.sender == _minter, "Unauthorised mint");
        _;
    }

    function mint(address to, uint256 amount) external OnlyMinter returns (bool) {
        _mint(to, amount);
        return true;
    }
}

contract AirVault {
    FudToken public immutable _fud_token;

    // address => amount
    mapping(address => uint256) _deposits_amount;

    event Deposited(address indexed account, uint indexed block_num, uint256 amount);
    event Withdrawn(address indexed account, uint indexed block_num, uint256 amount);

    constructor(address fud) {
        _fud_token = FudToken(fud);
    }

    function deposit(uint256 amount) external returns (bool) {
        // record deposit
        require(
            // make sure msg.sender have set enough allowence
            _fud_token.transferFrom(msg.sender, address(this), amount) == true,
            "Cannot transfer from depositer address to vault address"
        );
        _deposits_amount[msg.sender] += amount;

        // emit event and ends
        emit Deposited(msg.sender, amount, block.number);
        return true;
    }

    // this will withdraw the amount from deposits
    // now this amount would no longer be taken into account
    // advised to call claim() beforehand
    function withdraw(uint256 amount) public returns (bool) {
        require(_deposits_amount[msg.sender] > amount, "Given amount is not locked");
        
        require(
            _fud_token.transfer(msg.sender, amount),
            "Unexpected: cannot transfer from Airvault to depositer"
        );
        _deposits_amount[msg.sender] -= amount;

        emit Withdrawn(msg.sender, amount, block.number);
        return true;
    }

    function lockedBalanceOf(address account) external view returns (uint256) {
        return _deposits_amount[account];
    }
}
