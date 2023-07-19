import numpy as np
import pandas as pd
from scipy.optimize import linprog


def minimize_l1_norm(A, b):
    """
    Minimizes the L1 norm of a solution to a system of linear equations Ax = b.

    Parameters:
        A (numpy.ndarray): Coefficient matrix of the system.
        b (numpy.ndarray): Right-hand side vector of the system.

    Returns:
        numpy.ndarray: Solution vector that minimizes the L1 norm.
    """
    num_unknowns = A.shape[1]  # Number of unknowns

    # Define the objective function coefficients as 1's
    c = np.ones(num_unknowns)

    # Define the equality constraint matrix
    A_eq = A
    b_eq = b

    # Define the inequality constraint matrix
    A_ub = np.ones((1, num_unknowns))
    b_ub = np.array([0])

    # Define the bounds for the solution variables (-inf, inf)
    bounds = [(-10, 10)] * num_unknowns

    # Solve the linear programming problem
    res = linprog(c, A_eq=A_eq, b_eq=b_eq, bounds=bounds,
                  method='highs-ipm',
                  options={'presolve': False, 'maxiter': 1000})
    # Return the solution vector
    return res.x


data = pd.read_csv('out.csv')
A = data.drop('equals', axis=1)
b = data['equals']

r = minimize_l1_norm(A, b)
for x in range(r.shape[0]):
    print(f'{x}, {r[x]}')
