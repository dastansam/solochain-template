License: MIT-0

## Membership pallet

Some design choices and implementation details of the membership pallet.

### Storage types

- `Clubs` -> `StorageMap<ClubId, Club>`: Maps a club ID to a `Club` struct which contains some metadata about it, like owner, membership fee, etc.
- `ClubMembers` -> `StorageDoubleMap<ClubId, AccountId, ClubMemberMetadata>`: Maps a club ID and member's `AccountId` and some metadata about the member, like membership status, etc. `AccountId` is used as unique identifier for members, so there is no way to have duplicate members in a club. And it is more convenient to use this structure, instead of say: `StorageMap<ClubId, Vec<ClubMemberMetadata>>`. Because, if we were to look for certain member in the storage, we would have to load all members of the club and iterate over them to find the one we are looking for. With `StorageDoubleMap`, we can directly access the metadata of a member by providing club ID and member's `AccountId`. And at the same time, we can easily iterate over the keys without loading the values, which is a lot more efficient especially with large number of members.

### Member activity

Members' membership can expire, so we might think that we need some kind of on chain hook that toggles the membership status of members. We can use on-chain hooks of Substrate for this, but it introduces a lot of complexity and is arguably the most vulnerable part of the pallet. There is a need to keep track of the weight it uses, some kind of cursor to keep track of the last member that was checked, etc. 

Instead, we can just rely on offchain DApps or users to check the membership status of members with simply comparing the current block number with the block number when the member's membership expires. This way, we can keep the pallet simple and efficient.

### Membership pallet fees pool

Club creation deposit fee and all the membership fees go to the protocol, i.e simply transferred to the account derived from the membership pallet name. Root has access to withdraw from this pool.

In the future, we can make `T::ClubId` convertible to `T::AccountId` and derive deterministic addresses for each club. And we can use that account to store the membership fees, with a separate extrinsic for owners to withdraw from this account.
