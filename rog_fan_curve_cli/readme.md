
# Usage

```sh
rog_fan_curve '30c:56%,49c:56%,59c:56%,69c:56%,79c:56%,89c:56%,99c:56%,109c:56%'
```

The cpu and gpu fans can be controlled individually by adding the `--cpu` or `--gpu` flags.

You can override the detected board with the `--board` parameter e.g. `--board GA401IV`.

# Testing a new board

* Turn both fans to low.
```sh
rog_fan_curve_cli --board GA401IV '30c:1%,49c:1%,59c:1%,69c:1%,79c:1%,89c:1%,99c:1%,109c:1%'
```

* Turn up only the cpu fan and make sure one fan spins up.
```sh
rog_fan_curve_cli --board GA401IV --cpu '30c:50%,49c:50%,59c:50%,69c:50%,79c:50%,89c:50%,99c:50%,109c:50%'
```

* Turn up only the gpu fan and make sure only the other fan spins up.
```sh
rog_fan_curve_cli --board GA401IV '30c:1%,49c:1%,59c:1%,69c:1%,79c:1%,89c:1%,99c:1%,109c:1%'
rog_fan_curve_cli --board GA401IV --gpu '30c:50%,49c:50%,59c:50%,69c:50%,79c:50%,89c:50%,99c:50%,109c:50%'
```

[Open an issue](https://github.com/Yarn/rog_fan_curve/issues/new)
asking for a new board to be added.
Include the output of `cat /sys/class/dmi/id/board_name`.
