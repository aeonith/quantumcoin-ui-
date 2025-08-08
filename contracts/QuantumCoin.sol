// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address recipient, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

contract QuantumCoin is IERC20 {
    string public constant name = "QuantumCoin";
    string public constant symbol = "QTC";
    uint8 public constant decimals = 8;
    uint256 public constant totalSupply = 22_000_000 * 10**decimals; // 22 million QTC
    
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    
    address public owner;
    bool public tradingEnabled = false;
    
    // Quantum-resistant features (placeholder for future implementation)
    mapping(address => bool) public quantumResistantAddresses;
    
    event TradingEnabled();
    event QuantumResistantAddressAdded(address indexed account);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not the owner");
        _;
    }
    
    modifier tradingIsEnabled() {
        require(tradingEnabled || msg.sender == owner, "Trading not enabled");
        _;
    }
    
    constructor() {
        owner = msg.sender;
        _balances[owner] = totalSupply;
        emit Transfer(address(0), owner, totalSupply);
    }
    
    function balanceOf(address account) public view override returns (uint256) {
        return _balances[account];
    }
    
    function transfer(address recipient, uint256 amount) public override tradingIsEnabled returns (bool) {
        _transfer(msg.sender, recipient, amount);
        return true;
    }
    
    function allowance(address owner_, address spender) public view override returns (uint256) {
        return _allowances[owner_][spender];
    }
    
    function approve(address spender, uint256 amount) public override returns (bool) {
        _approve(msg.sender, spender, amount);
        return true;
    }
    
    function transferFrom(address sender, address recipient, uint256 amount) public override tradingIsEnabled returns (bool) {
        uint256 currentAllowance = _allowances[sender][msg.sender];
        require(currentAllowance >= amount, "ERC20: transfer amount exceeds allowance");
        
        _transfer(sender, recipient, amount);
        _approve(sender, msg.sender, currentAllowance - amount);
        
        return true;
    }
    
    function _transfer(address sender, address recipient, uint256 amount) internal {
        require(sender != address(0), "ERC20: transfer from the zero address");
        require(recipient != address(0), "ERC20: transfer to the zero address");
        require(_balances[sender] >= amount, "ERC20: transfer amount exceeds balance");
        
        _balances[sender] -= amount;
        _balances[recipient] += amount;
        
        emit Transfer(sender, recipient, amount);
    }
    
    function _approve(address owner_, address spender, uint256 amount) internal {
        require(owner_ != address(0), "ERC20: approve from the zero address");
        require(spender != address(0), "ERC20: approve to the zero address");
        
        _allowances[owner_][spender] = amount;
        emit Approval(owner_, spender, amount);
    }
    
    // Owner functions
    function enableTrading() external onlyOwner {
        require(!tradingEnabled, "Trading already enabled");
        tradingEnabled = true;
        emit TradingEnabled();
    }
    
    function addQuantumResistantAddress(address account) external onlyOwner {
        quantumResistantAddresses[account] = true;
        emit QuantumResistantAddressAdded(account);
    }
    
    function renounceOwnership() external onlyOwner {
        owner = address(0);
    }
    
    // Emergency functions (similar to RevStop in your blockchain)
    function emergencyPause() external onlyOwner {
        tradingEnabled = false;
    }
    
    // View functions
    function getCirculatingSupply() external pure returns (uint256) {
        return totalSupply; // All tokens are in circulation
    }
    
    function isQuantumResistant(address account) external view returns (bool) {
        return quantumResistantAddresses[account];
    }
}
