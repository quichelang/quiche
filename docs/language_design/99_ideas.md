## Ideas

Some ideas and thoughts on what may or may not become features in Quiche.

- automatic casting, but only using auto()
```python

y: f32 = 12.5
x: i32 = auto(y)
z: i32 = y # should result in compilation
```