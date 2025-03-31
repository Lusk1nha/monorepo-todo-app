import { SignInForm } from '@/components/forms/signin-form'
import { AuthContent } from '../../__components/auth-content'
import { AuthHeader } from '../../__components/auth-header'
import { PATHS } from '@/path'

export default function SignInPage() {
  return (
    <AuthContent>
      <AuthHeader
        link={{
          href: PATHS.SIGNUP,
          text: 'Sign up',
        }}
        title="Login to your account"
        subtitle="Don’t have an account?"
      />

      <SignInForm />
    </AuthContent>
  )
}
