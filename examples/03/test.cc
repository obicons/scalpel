// n^p
// power(x, 2)
// power: u -> (p: nat) -> u^p
int power(int n, int p) {
    int result = 1;
    while (p--)
        result *= n;
    return result;
}
