pragma solidity >=0.8.2 <0.9.0;
import "hardhat/console.sol";


contract Roulette {


    struct Table {
        mapping(uint => Bet) bets;
        uint totalBets;
        uint createdAt;
        uint nextTable;
    }

    struct TableView {
        uint totalBets;
        uint createdAt;
        uint nextTable;
        int8 result;
        Bet[] bets;
    }

    struct Bet {
        int8 position;
        address user;
        uint256 amount;
    }

    address house;
    mapping(address => uint) balances;
    Table[]  tables;
    mapping (uint => int8) tableResults;

    // modifier to  only allow the house (the owner) to call the function
    modifier isHouse() {
        require(msg.sender == house, "Caller is not house");
        _;
    }

    constructor() payable  {
        require(msg.value >= 50 ether, "At least fund 50 ether to create casino");
        house = msg.sender;
    }

    function createTable() public returns(uint) {
        Table storage newTable = tables.push();
        newTable.totalBets = 0;
        newTable.createdAt = block.timestamp;
        tableResults[tables.length - 1] = -1; 
        return tables.length - 1;
    }

    function bet(uint256 tableId, int8[] memory position, uint256[] memory amount) public  {
        require(tableResults[tableId] == -1, "Already spinned");
        require(position.length == amount.length, "Amounts and positions mismatch");

        uint256 totalAmount = 0;
        for (uint i=0; i < amount.length; i++) {
            totalAmount += amount[i];
        }

        require(balances[msg.sender] >= totalAmount, "Insufficient balance");

        for (uint256 i = 0; i < position.length; i++) {
            uint newBetId = tables[tableId].totalBets++;
            tables[tableId].bets[newBetId] = Bet({position: position[i], user: msg.sender, amount: amount[i]});
        }

        // Transfer balance to house
        balances[msg.sender] -= totalAmount;
        
    }

    function spin(uint256 tableId) public  {
        Table storage table = tables[tableId];
        require(tableResults[tableId] == -1, "Already spinned");
        require(tables[tableId].totalBets > 0, "Has no bets"); 
        require((block.timestamp - tables[tableId].createdAt) > 30, "Must wait for 30 second"); 


        bool isBetter = false;
        for (uint i = 0; i < tables[tableId].totalBets; i++) {
            if (tables[tableId].bets[i].user == msg.sender) {
                isBetter = true;
                i = tables[tableId].totalBets;
                break;
            }
        }
        require(isBetter, "Must be better");

        // random number between [0-36]
        tableResults[tableId] = int8(uint8(uint(keccak256(abi.encodePacked(block.timestamp,msg.sender))) % 37));

        for (uint i = 0; i < tables[tableId].totalBets; i++) {
            Bet memory _bet = table.bets[i];
            // If won between [0-36]
            if (_bet.position == tableResults[tableId]) {
                balances[_bet.user] += _bet.amount * 36; 
            }

            // TODO: Assign positions to numbers > 37
        }

        // TODO: emit spinned event
        // Create next table after spin
        table.nextTable = createTable();
    }

    function getTables() public view returns (int8[] memory) {
        int8[] memory result = new int8[](tables.length);
        for (uint i=0; i<tables.length; i++) {
            result[i] = tableResults[i];
        }
        return result;
    }

    function getTableById(uint tableId) public view returns (TableView memory) {
        uint totalBets = tables[tableId].totalBets;
        Bet[] memory bets = new Bet[](totalBets);
        for (uint i=0; i<tables[tableId].totalBets; i++) {
            bets[i] = tables[tableId].bets[i];
        }

        TableView memory tableView = TableView({
            totalBets: tables[tableId].totalBets,
            createdAt: tables[tableId].createdAt,
            nextTable: tables[tableId].nextTable,
            result: tableResults[tableId],
            bets: bets
        });

        return tableView;
    }

    
    function getBalance() public view returns(uint256) {
        return balances[msg.sender];
    }

    // Deposit money and get chips
    function exchange() public payable {
        balances[msg.sender] += msg.value;
    }

    // Withdraw money and give chips
    function cashin() public {
        address payable to = payable(msg.sender);
        to.transfer(balances[msg.sender]);
        balances[msg.sender] = 0;
    }


    // Put money to house safe
    function fund() public payable isHouse {
    }

    // Withdraw money from house safe 
    function withdraw() public isHouse {
        address payable to = payable(msg.sender);
        to.transfer(balances[msg.sender]);
    }

    // Get balane in house safe
    function checkSafe() public isHouse view returns(uint) {
        return address(this).balance;
    }

    function changeHouse(address newHouse) public isHouse {
        house = newHouse;
    }
    
    function getHouse() external view returns (address)  {
        return house;
    }
}   