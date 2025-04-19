# Setting Up Automated Deployment with GitHub Actions

This guide will help you set up automated deployments to Cloudflare Workers whenever you push to your GitHub repository.

## Step 1: Create a Cloudflare API Token

1. Log in to your Cloudflare dashboard at [dash.cloudflare.com](https://dash.cloudflare.com)
2. Navigate to "My Profile" > "API Tokens"
3. Click "Create Token"
4. Choose "Create Custom Token"
5. Give your token a name like "GitHub Actions Deployment"
6. Under "Permissions", add the following:
   - Account > Workers Scripts > Edit
   - Account > Workers Routes > Edit
   - Account > Account Settings > Read (needed for account verification)
   - Zone > Zone Settings > Read (needed for zone verification if you're using custom domains)
7. Under "Account Resources", select your account
8. Under "Zone Resources", select either "All zones" or specific zones if you prefer
9. Click "Continue to Summary" and then "Create Token"
10. **IMPORTANT**: Copy the token immediately and keep it secure. You will NOT be able to view it again.

## Step 2: Get Your Cloudflare Account ID

1. Log in to your Cloudflare dashboard
2. Your Account ID is displayed on the right sidebar on the Workers or Overview page
3. It should look something like: `a1b2c3d4e5f6g7h8i9j0`

## Step 3: Add Secrets to Your GitHub Repository

1. Go to your GitHub repository
2. Navigate to "Settings" > "Secrets and variables" > "Actions"
3. Click "New repository secret"
4. Create two secrets:
   - Name: `CF_API_TOKEN` - Value: your Cloudflare API token from Step 1
   - Name: `CF_ACCOUNT_ID` - Value: your Cloudflare Account ID from Step 2
5. Click "Add secret" for each

## Step 4: Push Your Code

The GitHub Action is now configured! The next time you push to the `main` branch, your Cloudflare Worker will be automatically deployed.

## Verification and Troubleshooting

1. After pushing to main, go to the "Actions" tab in your GitHub repository
2. You should see your workflow running
3. If the deployment is successful, you'll see a green checkmark
4. If there's an error, you can click on the workflow run to see detailed logs

If you encounter issues:
- Verify your API token has the correct permissions
- Check that your account ID is correct
- Ensure your wrangler.toml file is properly configured
- Review the GitHub Actions logs for specific error messages 