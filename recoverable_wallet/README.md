# Recoverable Wallet 

 This tutorial creates a smart contract that behaves like a personal wallet with some additional features on top, demonstrating the concept of account abstraction. Some features enabled by this concept include:  
  - Social recovery using trusted addresses to recover the account in case you lost it
  - Daily transaction limits
  - Allow lists for transfers exceding a given amount of tokens 
  
 In this example we implement the social recovery feature. Where a user can set a list of trusted addresses (`recovery_guardians`) that in case of a lost key to this wallet can recover the funds and transfer them to a new account. 

 [To the tutorial](tutorial.md)
