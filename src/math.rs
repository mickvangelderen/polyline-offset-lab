use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone)]
pub struct Point<A>(pub A);

impl Point<[f64; 2]> {
    pub fn normal(a: Point<[f64; 2]>, b: Point<[f64; 2]>) -> Vector<[f64; 2]> {
        let d = b - a;

        Vector([-d[1], d[0]]).normalize()
    }
}

impl<const N: usize> Vector<[f64; N]> {
    pub fn normalize(self) -> Self {
        let scale = 1.0 / self.magnitude();
        self * scale
    }

    pub fn magnitude(&self) -> f64 {
        self.0.iter().copied().map(|x| x * x).sum::<f64>().sqrt()
    }
}

#[inline(always)]
fn element_wise_binary<T, U, F, const N: usize>(a: [T; N], b: [T; N], f: F) -> [U; N]
where
    F: Fn(T, T) -> U,
    T: Copy,
{
    core::array::from_fn(|i| f(a[i], b[i]))
}

impl<T, const N: usize> Index<usize> for Point<[T; N]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for Point<[T; N]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T, const N: usize> Add<Vector<[T; N]>> for Point<[T; N]>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Vector<[T; N]>) -> Self::Output {
        Self(element_wise_binary(self.0, rhs.0, Add::add))
    }
}

impl<T, const N: usize> AddAssign<Vector<[T; N]>> for Point<[T; N]>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Vector<[T; N]>) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs += rhs;
        }
    }
}

impl<T, const N: usize> Sub<Vector<[T; N]>> for Point<[T; N]>
where
    T: Sub + Copy,
{
    type Output = Point<[<T as Sub>::Output; N]>;

    fn sub(self, rhs: Vector<[T; N]>) -> Self::Output {
        Point(element_wise_binary(self.0, rhs.0, Sub::sub))
    }
}

impl<T, const N: usize> SubAssign<Vector<[T; N]>> for Point<[T; N]>
where
    T: SubAssign,
{
    fn sub_assign(&mut self, rhs: Vector<[T; N]>) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs -= rhs;
        }
    }
}

impl<T, const N: usize> Sub for Point<[T; N]>
where
    T: Sub + Copy,
{
    type Output = Vector<[<T as Sub>::Output; N]>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector(element_wise_binary(self.0, rhs.0, Sub::sub))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector<A>(pub A);

impl<T, const N: usize> Index<usize> for Vector<[T; N]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for Vector<[T; N]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T, const N: usize> Add for Vector<[T; N]>
where
    T: Add + Copy,
{
    type Output = Vector<[<T as Add>::Output; N]>;

    fn add(self, rhs: Vector<[T; N]>) -> Self::Output {
        Vector(element_wise_binary(self.0, rhs.0, Add::add))
    }
}

impl<T, const N: usize> AddAssign for Vector<[T; N]>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Vector<[T; N]>) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs += rhs;
        }
    }
}

impl<T, const N: usize> Sub for Vector<[T; N]>
where
    T: Sub + Copy,
{
    type Output = Vector<[<T as Sub>::Output; N]>;

    fn sub(self, rhs: Vector<[T; N]>) -> Self::Output {
        Vector(element_wise_binary(self.0, rhs.0, Sub::sub))
    }
}

impl<T, const N: usize> SubAssign for Vector<[T; N]>
where
    T: SubAssign,
{
    fn sub_assign(&mut self, rhs: Vector<[T; N]>) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs -= rhs;
        }
    }
}

impl<T, const N: usize> Mul<T> for Vector<[T; N]>
where
    T: Mul + Copy,
{
    type Output = Vector<[<T as Mul>::Output; N]>;

    fn mul(self, rhs: T) -> Self::Output {
        Vector(core::array::from_fn(|i| self.0[i] * rhs))
    }
}

impl<T, const N: usize> MulAssign for Vector<[T; N]>
where
    T: MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs *= rhs;
        }
    }
}
