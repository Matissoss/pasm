<h1 align=center>rasm syntax</h1>

## assertions

This tutorial asserts that you have basic knowledge of x86-64 assembly.

## instructions

Instructions follow Intel-like syntax:
```
mnemonic destination, source
```

## immediates

immediates can be saved in `float` (32-bit), `double` (64-bit), `octal`, `binary`, `decimal` or `hex` formats prefixed with `$` (this applies to all numbers).

## registers

Registers are prefixed with `%` using known convention (rax, rbx, rcx).

## memory

Memory is a bit more complex. It follows this syntax:
```
(memory) !size
```

where `!size` can be one of these: `!byte` (for 8-bit value), `!word` (for 16-bit value), `!dword` (for 32-bit value) or `!qword` (for 64-bit value)

### base

```
(%register +- $displacement)
```

### index * scale

```
(%index * $scale +- displacement)
```

> [!WARN]
> `*` must be used, otherwise you will get error

### base + index * scale

```
(%base, %index * $scale +- $displacement)
```

## labels

Labels are created like this `name:`

## variables

> [!WARN]
> Currently working on

## globals

> [!WARN]
> Currently working on
