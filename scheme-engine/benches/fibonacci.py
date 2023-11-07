
import timeit


def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)


for n in [10, 20, 25]:
    ms = timeit.timeit(lambda: fibonacci(n), number=10000)
    print(f"fib({n}) -> {ms:.2f}ms")
