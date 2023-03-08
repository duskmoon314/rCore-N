#[doc = "Register `msr` reader"]
pub struct R(crate::R<MSR_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<MSR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<MSR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<MSR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `msr` writer"]
pub struct W(crate::W<MSR_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<MSR_SPEC>;
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
impl From<crate::W<MSR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<MSR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `dcts` reader - Delta Clear to Send"]
pub type DCTS_R = crate::BitReader<DCTS_A>;
#[doc = "Delta Clear to Send\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DCTS_A {
    #[doc = "0: `0`"]
    NO_CHANGE = 0,
    #[doc = "1: `1`"]
    CHANGE = 1,
}
impl From<DCTS_A> for bool {
    #[inline(always)]
    fn from(variant: DCTS_A) -> Self {
        variant as u8 != 0
    }
}
impl DCTS_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DCTS_A {
        match self.bits {
            false => DCTS_A::NO_CHANGE,
            true => DCTS_A::CHANGE,
        }
    }
    #[doc = "Checks if the value of the field is `NO_CHANGE`"]
    #[inline(always)]
    pub fn is_no_change(&self) -> bool {
        *self == DCTS_A::NO_CHANGE
    }
    #[doc = "Checks if the value of the field is `CHANGE`"]
    #[inline(always)]
    pub fn is_change(&self) -> bool {
        *self == DCTS_A::CHANGE
    }
}
#[doc = "Field `dcts` writer - Delta Clear to Send"]
pub type DCTS_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, DCTS_A, O>;
impl<'a, const O: u8> DCTS_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn no_change(self) -> &'a mut W {
        self.variant(DCTS_A::NO_CHANGE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn change(self) -> &'a mut W {
        self.variant(DCTS_A::CHANGE)
    }
}
#[doc = "Field `ddsr` reader - Delta Data Set Ready"]
pub type DDSR_R = crate::BitReader<DDSR_A>;
#[doc = "Delta Data Set Ready\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DDSR_A {
    #[doc = "0: `0`"]
    NO_CHANGE = 0,
    #[doc = "1: `1`"]
    CHANGE = 1,
}
impl From<DDSR_A> for bool {
    #[inline(always)]
    fn from(variant: DDSR_A) -> Self {
        variant as u8 != 0
    }
}
impl DDSR_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DDSR_A {
        match self.bits {
            false => DDSR_A::NO_CHANGE,
            true => DDSR_A::CHANGE,
        }
    }
    #[doc = "Checks if the value of the field is `NO_CHANGE`"]
    #[inline(always)]
    pub fn is_no_change(&self) -> bool {
        *self == DDSR_A::NO_CHANGE
    }
    #[doc = "Checks if the value of the field is `CHANGE`"]
    #[inline(always)]
    pub fn is_change(&self) -> bool {
        *self == DDSR_A::CHANGE
    }
}
#[doc = "Field `ddsr` writer - Delta Data Set Ready"]
pub type DDSR_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, DDSR_A, O>;
impl<'a, const O: u8> DDSR_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn no_change(self) -> &'a mut W {
        self.variant(DDSR_A::NO_CHANGE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn change(self) -> &'a mut W {
        self.variant(DDSR_A::CHANGE)
    }
}
#[doc = "Field `teri` reader - Trailing Edge Ring Indicator"]
pub type TERI_R = crate::BitReader<TERI_A>;
#[doc = "Trailing Edge Ring Indicator\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TERI_A {
    #[doc = "0: `0`"]
    NO_CHANGE = 0,
    #[doc = "1: `1`"]
    CHANGE = 1,
}
impl From<TERI_A> for bool {
    #[inline(always)]
    fn from(variant: TERI_A) -> Self {
        variant as u8 != 0
    }
}
impl TERI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> TERI_A {
        match self.bits {
            false => TERI_A::NO_CHANGE,
            true => TERI_A::CHANGE,
        }
    }
    #[doc = "Checks if the value of the field is `NO_CHANGE`"]
    #[inline(always)]
    pub fn is_no_change(&self) -> bool {
        *self == TERI_A::NO_CHANGE
    }
    #[doc = "Checks if the value of the field is `CHANGE`"]
    #[inline(always)]
    pub fn is_change(&self) -> bool {
        *self == TERI_A::CHANGE
    }
}
#[doc = "Field `teri` writer - Trailing Edge Ring Indicator"]
pub type TERI_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, TERI_A, O>;
impl<'a, const O: u8> TERI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn no_change(self) -> &'a mut W {
        self.variant(TERI_A::NO_CHANGE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn change(self) -> &'a mut W {
        self.variant(TERI_A::CHANGE)
    }
}
#[doc = "Field `ddcd` reader - Delta Data Carrier Detect"]
pub type DDCD_R = crate::BitReader<DDCD_A>;
#[doc = "Delta Data Carrier Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DDCD_A {
    #[doc = "0: `0`"]
    NO_CHANGE = 0,
    #[doc = "1: `1`"]
    CHANGE = 1,
}
impl From<DDCD_A> for bool {
    #[inline(always)]
    fn from(variant: DDCD_A) -> Self {
        variant as u8 != 0
    }
}
impl DDCD_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DDCD_A {
        match self.bits {
            false => DDCD_A::NO_CHANGE,
            true => DDCD_A::CHANGE,
        }
    }
    #[doc = "Checks if the value of the field is `NO_CHANGE`"]
    #[inline(always)]
    pub fn is_no_change(&self) -> bool {
        *self == DDCD_A::NO_CHANGE
    }
    #[doc = "Checks if the value of the field is `CHANGE`"]
    #[inline(always)]
    pub fn is_change(&self) -> bool {
        *self == DDCD_A::CHANGE
    }
}
#[doc = "Field `ddcd` writer - Delta Data Carrier Detect"]
pub type DDCD_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, DDCD_A, O>;
impl<'a, const O: u8> DDCD_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn no_change(self) -> &'a mut W {
        self.variant(DDCD_A::NO_CHANGE)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn change(self) -> &'a mut W {
        self.variant(DDCD_A::CHANGE)
    }
}
#[doc = "Field `cts` reader - Line State of Clear To Send"]
pub type CTS_R = crate::BitReader<CTS_A>;
#[doc = "Line State of Clear To Send\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CTS_A {
    #[doc = "0: `0`"]
    DEASSERTED = 0,
    #[doc = "1: `1`"]
    ASSERTED = 1,
}
impl From<CTS_A> for bool {
    #[inline(always)]
    fn from(variant: CTS_A) -> Self {
        variant as u8 != 0
    }
}
impl CTS_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> CTS_A {
        match self.bits {
            false => CTS_A::DEASSERTED,
            true => CTS_A::ASSERTED,
        }
    }
    #[doc = "Checks if the value of the field is `DEASSERTED`"]
    #[inline(always)]
    pub fn is_deasserted(&self) -> bool {
        *self == CTS_A::DEASSERTED
    }
    #[doc = "Checks if the value of the field is `ASSERTED`"]
    #[inline(always)]
    pub fn is_asserted(&self) -> bool {
        *self == CTS_A::ASSERTED
    }
}
#[doc = "Field `cts` writer - Line State of Clear To Send"]
pub type CTS_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, CTS_A, O>;
impl<'a, const O: u8> CTS_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn deasserted(self) -> &'a mut W {
        self.variant(CTS_A::DEASSERTED)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn asserted(self) -> &'a mut W {
        self.variant(CTS_A::ASSERTED)
    }
}
#[doc = "Field `dsr` reader - Line State of Data Set Ready"]
pub type DSR_R = crate::BitReader<DSR_A>;
#[doc = "Line State of Data Set Ready\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DSR_A {
    #[doc = "0: `0`"]
    DEASSERTED = 0,
    #[doc = "1: `1`"]
    ASSERTED = 1,
}
impl From<DSR_A> for bool {
    #[inline(always)]
    fn from(variant: DSR_A) -> Self {
        variant as u8 != 0
    }
}
impl DSR_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DSR_A {
        match self.bits {
            false => DSR_A::DEASSERTED,
            true => DSR_A::ASSERTED,
        }
    }
    #[doc = "Checks if the value of the field is `DEASSERTED`"]
    #[inline(always)]
    pub fn is_deasserted(&self) -> bool {
        *self == DSR_A::DEASSERTED
    }
    #[doc = "Checks if the value of the field is `ASSERTED`"]
    #[inline(always)]
    pub fn is_asserted(&self) -> bool {
        *self == DSR_A::ASSERTED
    }
}
#[doc = "Field `dsr` writer - Line State of Data Set Ready"]
pub type DSR_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, DSR_A, O>;
impl<'a, const O: u8> DSR_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn deasserted(self) -> &'a mut W {
        self.variant(DSR_A::DEASSERTED)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn asserted(self) -> &'a mut W {
        self.variant(DSR_A::ASSERTED)
    }
}
#[doc = "Field `ri` reader - Line State of Ring Indicator"]
pub type RI_R = crate::BitReader<RI_A>;
#[doc = "Line State of Ring Indicator\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RI_A {
    #[doc = "0: `0`"]
    DEASSERTED = 0,
    #[doc = "1: `1`"]
    ASSERTED = 1,
}
impl From<RI_A> for bool {
    #[inline(always)]
    fn from(variant: RI_A) -> Self {
        variant as u8 != 0
    }
}
impl RI_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> RI_A {
        match self.bits {
            false => RI_A::DEASSERTED,
            true => RI_A::ASSERTED,
        }
    }
    #[doc = "Checks if the value of the field is `DEASSERTED`"]
    #[inline(always)]
    pub fn is_deasserted(&self) -> bool {
        *self == RI_A::DEASSERTED
    }
    #[doc = "Checks if the value of the field is `ASSERTED`"]
    #[inline(always)]
    pub fn is_asserted(&self) -> bool {
        *self == RI_A::ASSERTED
    }
}
#[doc = "Field `ri` writer - Line State of Ring Indicator"]
pub type RI_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, RI_A, O>;
impl<'a, const O: u8> RI_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn deasserted(self) -> &'a mut W {
        self.variant(RI_A::DEASSERTED)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn asserted(self) -> &'a mut W {
        self.variant(RI_A::ASSERTED)
    }
}
#[doc = "Field `dcd` reader - Line State of Data Carrier Detect"]
pub type DCD_R = crate::BitReader<DCD_A>;
#[doc = "Line State of Data Carrier Detect\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DCD_A {
    #[doc = "0: `0`"]
    DEASSERTED = 0,
    #[doc = "1: `1`"]
    ASSERTED = 1,
}
impl From<DCD_A> for bool {
    #[inline(always)]
    fn from(variant: DCD_A) -> Self {
        variant as u8 != 0
    }
}
impl DCD_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DCD_A {
        match self.bits {
            false => DCD_A::DEASSERTED,
            true => DCD_A::ASSERTED,
        }
    }
    #[doc = "Checks if the value of the field is `DEASSERTED`"]
    #[inline(always)]
    pub fn is_deasserted(&self) -> bool {
        *self == DCD_A::DEASSERTED
    }
    #[doc = "Checks if the value of the field is `ASSERTED`"]
    #[inline(always)]
    pub fn is_asserted(&self) -> bool {
        *self == DCD_A::ASSERTED
    }
}
#[doc = "Field `dcd` writer - Line State of Data Carrier Detect"]
pub type DCD_W<'a, const O: u8> = crate::BitWriter<'a, u32, MSR_SPEC, DCD_A, O>;
impl<'a, const O: u8> DCD_W<'a, O> {
    #[doc = "`0`"]
    #[inline(always)]
    pub fn deasserted(self) -> &'a mut W {
        self.variant(DCD_A::DEASSERTED)
    }
    #[doc = "`1`"]
    #[inline(always)]
    pub fn asserted(self) -> &'a mut W {
        self.variant(DCD_A::ASSERTED)
    }
}
impl R {
    #[doc = "Bit 0 - Delta Clear to Send"]
    #[inline(always)]
    pub fn dcts(&self) -> DCTS_R {
        DCTS_R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Delta Data Set Ready"]
    #[inline(always)]
    pub fn ddsr(&self) -> DDSR_R {
        DDSR_R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Trailing Edge Ring Indicator"]
    #[inline(always)]
    pub fn teri(&self) -> TERI_R {
        TERI_R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Delta Data Carrier Detect"]
    #[inline(always)]
    pub fn ddcd(&self) -> DDCD_R {
        DDCD_R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Line State of Clear To Send"]
    #[inline(always)]
    pub fn cts(&self) -> CTS_R {
        CTS_R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Line State of Data Set Ready"]
    #[inline(always)]
    pub fn dsr(&self) -> DSR_R {
        DSR_R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Line State of Ring Indicator"]
    #[inline(always)]
    pub fn ri(&self) -> RI_R {
        RI_R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Line State of Data Carrier Detect"]
    #[inline(always)]
    pub fn dcd(&self) -> DCD_R {
        DCD_R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Delta Clear to Send"]
    #[inline(always)]
    #[must_use]
    pub fn dcts(&mut self) -> DCTS_W<0> {
        DCTS_W::new(self)
    }
    #[doc = "Bit 1 - Delta Data Set Ready"]
    #[inline(always)]
    #[must_use]
    pub fn ddsr(&mut self) -> DDSR_W<1> {
        DDSR_W::new(self)
    }
    #[doc = "Bit 2 - Trailing Edge Ring Indicator"]
    #[inline(always)]
    #[must_use]
    pub fn teri(&mut self) -> TERI_W<2> {
        TERI_W::new(self)
    }
    #[doc = "Bit 3 - Delta Data Carrier Detect"]
    #[inline(always)]
    #[must_use]
    pub fn ddcd(&mut self) -> DDCD_W<3> {
        DDCD_W::new(self)
    }
    #[doc = "Bit 4 - Line State of Clear To Send"]
    #[inline(always)]
    #[must_use]
    pub fn cts(&mut self) -> CTS_W<4> {
        CTS_W::new(self)
    }
    #[doc = "Bit 5 - Line State of Data Set Ready"]
    #[inline(always)]
    #[must_use]
    pub fn dsr(&mut self) -> DSR_W<5> {
        DSR_W::new(self)
    }
    #[doc = "Bit 6 - Line State of Ring Indicator"]
    #[inline(always)]
    #[must_use]
    pub fn ri(&mut self) -> RI_W<6> {
        RI_W::new(self)
    }
    #[doc = "Bit 7 - Line State of Data Carrier Detect"]
    #[inline(always)]
    #[must_use]
    pub fn dcd(&mut self) -> DCD_W<7> {
        DCD_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "UART Modem Status Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [msr](index.html) module"]
pub struct MSR_SPEC;
impl crate::RegisterSpec for MSR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [msr::R](R) reader structure"]
impl crate::Readable for MSR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [msr::W](W) writer structure"]
impl crate::Writable for MSR_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets msr to value 0"]
impl crate::Resettable for MSR_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
