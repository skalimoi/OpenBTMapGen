# OpenBattlesim Scenario Generator

This is a complementary project related to ![OpenBattleSim](https://github.com/skalimoi/OpenBattlesim) and consists of a standalone Scenario Generator that players will be able to use to create customized environments ready to use in OpenBattlesim.

It can generate 2^n sized maps from various types of noise and apply erosive forces to the terrain to make it realistic and generate water/river masks. Additionally, a Weather editor is also included, which enables users to specify custom weather parameters and view the prelimitary simulations on a 3D grid.

## Screenshots

![imagen](https://github.com/skalimoi/OpenBTMapGen/assets/53193415/1daef743-c881-4be8-9ad1-3743e052b14c)
![imagen](https://github.com/skalimoi/OpenBTMapGen/assets/53193415/df9680e2-6454-4746-be1c-db773af727bb)



## Development note

Currently only the topography systems (erosion, noise generation and visualization) are finished. The weather editor is on the works.

The erosion system (SimpleHydrology) was gracefully implemented by weigert on ![this repository](https://github.com/weigert/SimpleHydrology) and was ported to Rust ![here](https://github.com/skalimoi/SimpleHydrologyRust).
