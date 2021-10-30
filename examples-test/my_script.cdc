    pub struct User {
        pub var balance: UFix64
        pub var address: Address
        pub var name: String

        init(name: String, address: Address, balance: UFix64) {
            self.name = name
            self.address = address
            self.balance = balance
        }
    }

    pub fun main(name: String): User {
        return User(
            name: name,
            address: 0x1,
            balance: 10.0
        )
    }