# Election 

This tutorial will guide you through the creation of a voting smart contract using Odra. The contract will be built assuming the following principles:

* The deployer can specify candidates and a final voting time in the constructor.
* The final voting time is denominated in block height.
* The deployer cannot modify the candidates or end time after deployment.
* Any account, besides the deployer, may make one vote for any candidate they please.

The contract could be extended to allow for modifications to candidates, end time, and voting capabilities, but this tutorial avoids these functionalities in the interest of simplicity.

[To the tutorial](tutorial.md)
