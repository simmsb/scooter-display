pin_project_lite::pin_project! {
    #[repr(transparent)]
    pub struct NoInline<F> {
        #[pin]
        inner: F,
    }
}

impl<F> NoInline<F> {
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F: Future> Future for NoInline<F> {
    type Output = F::Output;

    #[inline(never)]
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

pub trait NoInlineFutExt: Sized {
    type Output;

    fn no_inline(self) -> NoInline<Self>;
}

impl<F: Future> NoInlineFutExt for F {
    type Output = F::Output;

    fn no_inline(self) -> NoInline<Self> {
        NoInline::new(self)
    }
}
