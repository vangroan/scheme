
import timeit


def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)


ms = timeit.timeit(lambda: fibonacci(20), number=10000)
print(f"{ms:.2f}ms")
