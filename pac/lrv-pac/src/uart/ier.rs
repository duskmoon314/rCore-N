#[doc = "Register `ier` reader"]
pub struct R(crate::R<IER_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<IER_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<IER_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<IER_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `ier` writer"]
pub struct W(crate::W<IER_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<IER_SPEC>;
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
impl From<crate::W<IER_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<IER_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `erbfi` reader - Enable Received Data Available Interrupt"]
pub type ERBFI_R = crate::BitReader<ERBFI_A>;
#[doc = "Enable Received Data Available Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ERBFI_A {
    #[doc = "0: `0`"]
    DISABLE = 0,
    #[doc = "1: `1`"]
    ENABLE = 1,
}
impl From<ERBFI_A> for bool {
    #[inline(always)]
    fn from(variant: ERBFI_A) -> Self {
        variant as u8 != 0
    }
}
impl ERBFI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> ERBFI_A {
        match self.bits {
            false => ERBFI_A::DISABLE,
            true => ERBFI_A::ENABLE,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ERBFI_A::DISABLE
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ERBFI_A::ENABLE
    }
}
#[doc = "Field `erbfi` writer - Enable Received Data Available Interrupt"]
pub type ERBFI_W<'a, const O: u8> = crate::BitWriter<'a, u32, IER_SPEC, ERBFI_A, O>;
impl<'a, const O: u8> ERBFI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(ERBFI_A::DISABLE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(ERBFI_A::ENABLE)
    }
}
#[doc = "Field `etbei` reader - Enable Transmit Holding Register Empty Interrupt"]
pub type ETBEI_R = crate::BitReader<ETBEI_A>;
#[doc = "Enable Transmit Holding Register Empty Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ETBEI_A {
    #[doc = "0: `0`"]
    DISABLE = 0,
    #[doc = "1: `1`"]
    ENABLE = 1,
}
impl From<ETBEI_A> for bool {
    #[inline(always)]
    fn from(variant: ETBEI_A) -> Self {
        variant as u8 != 0
    }
}
impl ETBEI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> ETBEI_A {
        match self.bits {
            false => ETBEI_A::DISABLE,
            true => ETBEI_A::ENABLE,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ETBEI_A::DISABLE
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ETBEI_A::ENABLE
    }
}
#[doc = "Field `etbei` writer - Enable Transmit Holding Register Empty Interrupt"]
pub type ETBEI_W<'a, const O: u8> = crate::BitWriter<'a, u32, IER_SPEC, ETBEI_A, O>;
impl<'a, const O: u8> ETBEI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(ETBEI_A::DISABLE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(ETBEI_A::ENABLE)
    }
}
#[doc = "Field `elsi` reader - Enable Receiver Line Status Interrupt"]
pub type ELSI_R = crate::BitReader<ELSI_A>;
#[doc = "Enable Receiver Line Status Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ELSI_A {
    #[doc = "0: `0`"]
    DISABLE = 0,
    #[doc = "1: `1`"]
    ENABLE = 1,
}
impl From<ELSI_A> for bool {
    #[inline(always)]
    fn from(variant: ELSI_A) -> Self {
        variant as u8 != 0
    }
}
impl ELSI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> ELSI_A {
        match self.bits {
            false => ELSI_A::DISABLE,
            true => ELSI_A::ENABLE,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ELSI_A::DISABLE
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ELSI_A::ENABLE
    }
}
#[doc = "Field `elsi` writer - Enable Receiver Line Status Interrupt"]
pub type ELSI_W<'a, const O: u8> = crate::BitWriter<'a, u32, IER_SPEC, ELSI_A, O>;
impl<'a, const O: u8> ELSI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(ELSI_A::DISABLE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(ELSI_A::ENABLE)
    }
}
#[doc = "Field `edssi` reader - Enable Modem Status Interrupt"]
pub type EDSSI_R = crate::BitReader<EDSSI_A>;
#[doc = "Enable Modem Status Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EDSSI_A {
    #[doc = "0: `0`"]
    DISABLE = 0,
    #[doc = "1: `1`"]
    ENABLE = 1,
}
impl From<EDSSI_A> for bool {
    #[inline(always)]
    fn from(variant: EDSSI_A) -> Self {
        variant as u8 != 0
    }
}
impl EDSSI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> EDSSI_A {
        match self.bits {
            false => EDSSI_A::DISABLE,
            true => EDSSI_A::ENABLE,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == EDSSI_A::DISABLE
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == EDSSI_A::ENABLE
    }
}
#[doc = "Field `edssi` writer - Enable Modem Status Interrupt"]
pub type EDSSI_W<'a, const O: u8> = crate::BitWriter<'a, u32, IER_SPEC, EDSSI_A, O>;
impl<'a, const O: u8> EDSSI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(EDSSI_A::DISABLE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(EDSSI_A::ENABLE)
    }
}
impl R {
    #[doc = "Bit 0 - Enable Received Data Available Interrupt"]
    #[inline(always)]
    pub fn erbfi(&self) -> ERBFI_R {
        ERBFI_R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enable Transmit Holding Register Empty Interrupt"]
    #[inline(always)]
    pub fn etbei(&self) -> ETBEI_R {
        ETBEI_R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Receiver Line Status Interrupt"]
    #[inline(always)]
    pub fn elsi(&self) -> ELSI_R {
        ELSI_R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enable Modem Status Interrupt"]
    #[inline(always)]
    pub fn edssi(&self) -> EDSSI_R {
        EDSSI_R::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Received Data Available Interrupt"]
    #[inline(always)]
    #[must_use]
    pub fn erbfi(&mut self) -> ERBFI_W<0> {
        ERBFI_W::new(self)
    }
    #[doc = "Bit 1 - Enable Transmit Holding Register Empty Interrupt"]
    #[inline(always)]
    #[must_use]
    pub fn etbei(&mut self) -> ETBEI_W<1> {
        ETBEI_W::new(self)
    }
    #[doc = "Bit 2 - Enable Receiver Line Status Interrupt"]
    #[inline(always)]
    #[must_use]
    pub fn elsi(&mut self) -> ELSI_W<2> {
        ELSI_W::new(self)
    }
    #[doc = "Bit 3 - Enable Modem Status Interrupt"]
    #[inline(always)]
    #[must_use]
    pub fn edssi(&mut self) -> EDSSI_W<3> {
        EDSSI_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "UART Interrupt Enable Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ier](index.html) module"]
pub struct IER_SPEC;
impl crate::RegisterSpec for IER_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [ier::R](R) reader structure"]
impl crate::Readable for IER_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [ier::W](W) writer structure"]
impl crate::Writable for IER_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets ier to value 0"]
impl crate::Resettable for IER_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
