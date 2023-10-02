spec aptos_framework::managed_coin {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    spec burn<CoinType>(
        account: &signer,
        amount: u64,
    ) {
        use aptos_framework::optional_aggregator;
        use aptos_std::type_info;
        use std::option;
        use aptos_framework::aggregator;

        let account_addr = signer::address_of(account);

        // Resource Capabilities<CoinType> should exists in the signer address.
        aborts_if !exists<Capabilities<CoinType>>(account_addr);
        let coin_store = global<coin::CoinStore<CoinType>>(account_addr);
        let balance = coin_store.coin.value;

        // Resource CoinStore<CoinType> should exists in the signer.
        aborts_if !exists<coin::CoinStore<CoinType>>(account_addr);

        // Account should not be frozen and should have sufficient balance.
        aborts_if coin_store.frozen;
        aborts_if balance < amount;

        let addr =  type_info::type_of<CoinType>().account_address;
        let maybe_supply = global<coin::CoinInfo<CoinType>>(addr).supply;
        // Ensure the amount won't be overflow.
        aborts_if amount <= 0;
        aborts_if !exists<coin::CoinInfo<CoinType>>(addr);
        aborts_if option::is_some(maybe_supply) && optional_aggregator::is_parallelizable(option::borrow(maybe_supply))
            && aggregator::spec_aggregator_get_val(option::borrow(option::borrow(maybe_supply).aggregator)) <
            amount;
        aborts_if option::is_some(maybe_supply) && !optional_aggregator::is_parallelizable(option::borrow(maybe_supply))
            && option::borrow(option::borrow(maybe_supply).integer).value <
            amount;

        // Ensure that the global 'supply' decreases by 'amount'.
        ensures coin::supply<CoinType> == old(coin::supply<CoinType>) - amount;
    }

    /// Make sure `name` and `symbol` are legal length.
    /// Only the creator of `CoinType` can initialize.
    /// The 'name' and 'symbol' should be valid utf8 bytes
    /// The Capabilities<CoinType> should not be under the signer before creating;
    /// The Capabilities<CoinType> should be under the signer after creating;
    spec initialize<CoinType>(
        account: &signer,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        monitor_supply: bool,
    ) {
        include coin::InitializeInternalSchema<CoinType>;
        aborts_if !string::spec_internal_check_utf8(name);
        aborts_if !string::spec_internal_check_utf8(symbol);
        aborts_if exists<Capabilities<CoinType>>(signer::address_of(account));
        ensures exists<Capabilities<CoinType>>(signer::address_of(account));
    }

    /// The Capabilities<CoinType> should not exist in the signer address.
    /// The `dst_addr` should not be frozen.
    spec mint<CoinType>(
        account: &signer,
        dst_addr: address,
        amount: u64,
    ) {
        use aptos_std::type_info;
        let account_addr = signer::address_of(account);

        // Capabilities<CoinType> should not exist in the signer address.
        aborts_if !exists<Capabilities<CoinType>>(account_addr);

        let addr = type_info::type_of<CoinType>().account_address;
        aborts_if (amount != 0) && !exists<coin::CoinInfo<CoinType>>(addr);

        let coin_store = global<coin::CoinStore<CoinType>>(dst_addr);
        // Capabilities<CoinType> should not exist in the destination address and `dst_addr` should not be frozen.
        aborts_if !exists<coin::CoinStore<CoinType>>(dst_addr);
        aborts_if coin_store.frozen;

        include coin::AbortsIfAggregatorAdd<CoinType>;

        // Ensure that the global 'supply' increases by 'amount'.
        ensures coin::supply<CoinType> == old(coin::supply<CoinType>) + amount;

        // Ensure that 'amount' in deposited at 'dst_addr'.
        ensures global<coin::CoinStore<CoinType>>(dst_addr).coin.value == old(global<coin::CoinStore<CoinType>>(dst_addr)).coin.value + amount;
    }

    /// An account can only be registered once.
    /// Updating `Account.guid_creation_num` will not overflow.
    spec register<CoinType>(account: &signer) {
        use aptos_framework::account;
        use aptos_std::type_info;

        let account_addr = signer::address_of(account);
        let acc = global<account::Account>(account_addr);

        aborts_if !exists<coin::CoinStore<CoinType>>(account_addr) && acc.guid_creation_num + 2 >= account::MAX_GUID_CREATION_NUM;
        aborts_if !exists<coin::CoinStore<CoinType>>(account_addr) && acc.guid_creation_num + 2 > MAX_U64;
        aborts_if !exists<coin::CoinStore<CoinType>>(account_addr) && !exists<account::Account>(account_addr);
        aborts_if !exists<coin::CoinStore<CoinType>>(account_addr) && !type_info::spec_is_struct<CoinType>();

        ensures exists<coin::CoinStore<CoinType>>(account_addr);
    }
}
