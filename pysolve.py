import pandas as pd
import numpy as np

data = pd.read_csv('out.csv')
A = data.drop('equals', axis=1)
b = data['equals']
x = np.linalg.lstsq(A, b)[0]
print(np.allclose(np.dot(A, x), b))
