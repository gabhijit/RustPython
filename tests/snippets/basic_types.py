# Spec: https://docs.python.org/2/library/types.html
print(None)
# TypeType
# print(True) # LOAD_NAME???
print(1)
# print(1L) # Long
print(1.1)
# ComplexType
print("abc")
# print(u"abc")
# Structural below
print((1, 2)) # Tuple can be any length, but fixed after declared
x = (1,2)
print(x[0]) # Tuple can be any length, but fixed after declared
print([1, 2, 3])
# print({"first":1,"second":2})

print(int(1))
print(int(1.2))
print(float(1))
print(float(1.2))

assert type(1 - 2) is int
assert type(2 / 3) is float
x = 1
assert type(x) is int
assert type(x - 1) is int

a = bytes([1, 2, 3])
print(a)
b = bytes([1, 2, 3])
assert a == b

try:
    bytes([object()])
except TypeError:
    pass

assert int() == 0
