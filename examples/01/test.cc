namespace n1 {
    double z1;

    class C {
        void cm() {}

        int c_x;
    };

    template <typename T>
    class C1 {
        template <typename U>
        void c1m();
        int c1_x;
    };

    template <typename T>
    template <typename U>
    void C1<T>::c1m() {
        int my_val;
    }
}

int main() {
    /// x: m
    /// frame(x) = (global, _)
    double x;

    /// z: cm
    /// frame(z) = (local, _)
    double z = x;
}
