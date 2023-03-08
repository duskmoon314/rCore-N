#[doc = "Register `lsr` reader"]
pub struct R(crate::R<LSR_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<LSR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<LSR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<LSR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `lsr` writer"]
pub struct W(crate::W<LSR_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<LSR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<LSR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<LSR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `dr` reader - Data Ready"]
pub type DR_R = crate::BitReader<DR_A>;
#[doc = "Data Ready\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DR_A {
    #[doc = "1: `1`"]
    READY = 1,
}
impl From<DR_A> for bool {
    #[inline(always)]
    fn from(variant: DR_A) -> Self {
        variant as u8 != 0
    }
}
impl DR_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<DR_A> {
        match self.bits {
            true => Some(DR_A::READY),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `READY`"]
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        *self == DR_A::READY
    }
}
#[doc = "Field `dr` writer - Data Ready"]
pub type DR_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, DR_A, O>;
impl<'a, const O: u8> DR_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn ready(self) -> &'a mut W {
        self.variant(DR_A::READY)
    }
}
#[doc = "Field `oe` reader - Overrun Error"]
pub type OE_R = crate::BitReader<OE_A>;
#[doc = "Overrun Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OE_A {
    #[doc = "1: `1`"]
    ERROR = 1,
}
impl From<OE_A> for bool {
    #[inline(always)]
    fn from(variant: OE_A) -> Self {
        variant as u8 != 0
    }
}
impl OE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<OE_A> {
        match self.bits {
            true => Some(OE_A::ERROR),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `ERROR`"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == OE_A::ERROR
    }
}
#[doc = "Field `oe` writer - Overrun Error"]
pub type OE_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, OE_A, O>;
impl<'a, const O: u8> OE_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn error(self) -> &'a mut W {
        self.variant(OE_A::ERROR)
    }
}
#[doc = "Field `pe` reader - Parity Error"]
pub type PE_R = crate::BitReader<PE_A>;
#[doc = "Parity Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PE_A {
    #[doc = "1: `1`"]
    ERROR = 1,
}
impl From<PE_A> for bool {
    #[inline(always)]
    fn from(variant: PE_A) -> Self {
        variant as u8 != 0
    }
}
impl PE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<PE_A> {
        match self.bits {
            true => Some(PE_A::ERROR),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `ERROR`"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == PE_A::ERROR
    }
}
#[doc = "Field `pe` writer - Parity Error"]
pub type PE_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, PE_A, O>;
impl<'a, const O: u8> PE_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn error(self) -> &'a mut W {
        self.variant(PE_A::ERROR)
    }
}
#[doc = "Field `fe` reader - Framing Error"]
pub type FE_R = crate::BitReader<FE_A>;
#[doc = "Framing Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FE_A {
    #[doc = "1: `1`"]
    ERROR = 1,
}
impl From<FE_A> for bool {
    #[inline(always)]
    fn from(variant: FE_A) -> Self {
        variant as u8 != 0
    }
}
impl FE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<FE_A> {
        match self.bits {
            true => Some(FE_A::ERROR),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `ERROR`"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == FE_A::ERROR
    }
}
#[doc = "Field `fe` writer - Framing Error"]
pub type FE_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, FE_A, O>;
impl<'a, const O: u8> FE_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn error(self) -> &'a mut W {
        self.variant(FE_A::ERROR)
    }
}
#[doc = "Field `bi` reader - Break Interrupt"]
pub type BI_R = crate::BitReader<bool>;
#[doc = "Field `bi` writer - Break Interrupt"]
pub type BI_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, bool, O>;
#[doc = "Field `thre` reader - TX Holding Register Empty"]
pub type THRE_R = crate::BitReader<THRE_A>;
#[doc = "TX Holding Register Empty\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum THRE_A {
    #[doc = "1: `1`"]
    EMPTY = 1,
}
impl From<THRE_A> for bool {
    #[inline(always)]
    fn from(variant: THRE_A) -> Self {
        variant as u8 != 0
    }
}
impl THRE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<THRE_A> {
        match self.bits {
            true => Some(THRE_A::EMPTY),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `EMPTY`"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == THRE_A::EMPTY
    }
}
#[doc = "Field `thre` writer - TX Holding Register Empty"]
pub type THRE_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, THRE_A, O>;
impl<'a, const O: u8> THRE_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn empty(self) -> &'a mut W {
        self.variant(THRE_A::EMPTY)
    }
}
#[doc = "Field `temt` reader - Transmitter Empty"]
pub type TEMT_R = crate::BitReader<TEMT_A>;
#[doc = "Transmitter Empty\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TEMT_A {
    #[doc = "1: `1`"]
    EMPTY = 1,
}
impl From<TEMT_A> for bool {
    #[inline(always)]
    fn from(variant: TEMT_A) -> Self {
        variant as u8 != 0
    }
}
impl TEMT_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<TEMT_A> {
        match self.bits {
            true => Some(TEMT_A::EMPTY),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `EMPTY`"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == TEMT_A::EMPTY
    }
}
#[doc = "Field `temt` writer - Transmitter Empty"]
pub type TEMT_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, TEMT_A, O>;
impl<'a, const O: u8> TEMT_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn empty(self) -> &'a mut W {
        self.variant(TEMT_A::EMPTY)
    }
}
#[doc = "Field `fifoerr` reader - RX Data Error in FIFO"]
pub type FIFOERR_R = crate::BitReader<FIFOERR_A>;
#[doc = "RX Data Error in FIFO\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FIFOERR_A {
    #[doc = "1: `1`"]
    ERROR = 1,
}
impl From<FIFOERR_A> for bool {
    #[inline(always)]
    fn from(variant: FIFOERR_A) -> Self {
        variant as u8 != 0
    }
}
impl FIFOERR_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<FIFOERR_A> {
        match self.bits {
            true => Some(FIFOERR_A::ERROR),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `ERROR`"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == FIFOERR_A::ERROR
    }
}
#[doc = "Field `fifoerr` writer - RX Data Error in FIFO"]
pub type FIFOERR_W<'a, const O: u8> = crate::BitWriter<'a, u32, LSR_SPEC, FIFOERR_A, O>;
impl<'a, const O: u8> FIFOERR_W<'a, O> {
    #[doc = "`1`"]
    #[inline(always)]
    pub fn error(self) -> &'a mut W {
        self.variant(FIFOERR_A::ERROR)
    }
}
impl R {
    #[doc = "Bit 0 - Data Ready"]
    #[inline(always)]
    pub fn dr(&self) -> DR_R {
        DR_R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Overrun Error"]
    #[inline(always)]
    pub fn oe(&self) -> OE_R {
        OE_R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Parity Error"]
    #[inline(always)]
    pub fn pe(&self) -> PE_R {
        PE_R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Framing Error"]
    #[inline(always)]
    pub fn fe(&self) -> FE_R {
        FE_R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Break Interrupt"]
    #[inline(always)]
    pub fn bi(&self) -> BI_R {
        BI_R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - TX Holding Register Empty"]
    #[inline(always)]
    pub fn thre(&self) -> THRE_R {
        THRE_R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Transmitter Empty"]
    #[inline(always)]
    pub fn temt(&self) -> TEMT_R {
        TEMT_R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - RX Data Error in FIFO"]
    #[inline(always)]
    pub fn fifoerr(&self) -> FIFOERR_R {
        FIFOERR_R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Data Ready"]
    #[inline(always)]
    #[must_use]
    pub fn dr(&mut self) -> DR_W<0> {
        DR_W::new(self)
    }
    #[doc = "Bit 1 - Overrun Error"]
    #[inline(always)]
    #[must_use]
    pub fn oe(&mut self) -> OE_W<1> {
        OE_W::new(self)
    }
    #[doc = "Bit 2 - Parity Error"]
    #[inline(always)]
    #[must_use]
    pub fn pe(&mut self) -> PE_W<2> {
        PE_W::new(self)
    }
    #[doc = "Bit 3 - Framing Error"]
    #[inline(always)]
    #[must_use]
    pub fn fe(&mut self) -> FE_W<3> {
        FE_W::new(self)
    }
    #[doc = "Bit 4 - Break Interrupt"]
    #[inline(always)]
    #[must_use]
    pub fn bi(&mut self) -> BI_W<4> {
        BI_W::new(self)
    }
    #[doc = "Bit 5 - TX Holding Register Empty"]
    #[inline(always)]
    #[must_use]
    pub fn thre(&mut self) -> THRE_W<5> {
        THRE_W::new(self)
    }
    #[doc = "Bit 6 - Transmitter Empty"]
    #[inline(always)]
    #[must_use]
    pub fn temt(&mut self) -> TEMT_W<6> {
        TEMT_W::new(self)
    }
    #[doc = "Bit 7 - RX Data Error in FIFO"]
    #[inline(always)]
    #[must_use]
    pub fn fifoerr(&mut self) -> FIFOERR_W<7> {
        FIFOERR_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "UART Line Status Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lsr](index.html) module"]
pub struct LSR_SPEC;
impl crate::RegisterSpec for LSR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [lsr::R](R) reader structure"]
impl crate::Readable for LSR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [lsr::W](W) writer structure"]
impl crate::Writable for LSR_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets lsr to value 0"]
impl crate::Resettable for LSR_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
