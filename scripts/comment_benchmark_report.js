module.exports = async ({github, context}) => {
    const comment = process.env.COMMENT_BODY;
  
    // Find existing benchmark comment
    const { data: comments } = await github.rest.issues.listComments({
      owner: context.repo.owner,
      repo: context.repo.repo,
      issue_number: context.issue.number,
    });
  
    const botComment = comments.find(comment => 
      comment.user.type === 'Bot' && 
      comment.body.includes('📊 Benchmark Results')
    );
  
    if (botComment) {
      // Update existing comment
      await github.rest.issues.updateComment({
        owner: context.repo.owner,
        repo: context.repo.repo,
        comment_id: botComment.id,
        body: comment
      });
    } else {
      // Create new comment
      await github.rest.issues.createComment({
        owner: context.repo.owner,
        repo: context.repo.repo,
        issue_number: context.issue.number,
        body: comment
      });
    }
  };