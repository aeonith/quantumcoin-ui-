document.querySelectorAll('.section').forEach(sec=>{
  new IntersectionObserver(([e])=>{
    if(e.isIntersecting) e.target.classList.add('reveal');
  },{threshold:.2}).observe(sec);
});

function switchMode(){
  const title=document.getElementById('authTitle');
  const btn=document.getElementById('authBtn');
  const confirm=document.getElementById('confirm');
  const alt=document.getElementById('altText');
  const signup=title.textContent==='Sign Up';
  title.textContent=signup?'Login':'Sign Up';
  btn.textContent=signup?'Login':'Create Account';
  confirm.style.display=signup?'none':'block';
  alt.textContent=signup?'Sign Up':'Login';
}

let shownBalance=0;
function updateBalance(newBal){
  const balEl=document.getElementById('bal');
  const diff=newBal-shownBalance;
  const steps=20, increment=diff/steps, interval=30;
  let i=0;clearInterval(balEl.timer);
  balEl.timer=setInterval(()=>{
    if(i++>=steps){clearInterval(balEl.timer);shownBalance=newBal;balEl.textContent=newBal;return;}
    shownBalance+=increment;balEl.textContent=shownBalance.toFixed(2);
  },interval);
}

// Fake initial load
setTimeout(()=>updateBalance(125.00),1200);

function mine(){
  const btn=document.querySelector('.mineGlow');
  btn.innerText='Mining…';
  btn.disabled=true;
  setTimeout(()=>{
    btn.innerText='Mine';
    btn.disabled=false;
    updateBalance(shownBalance+50);
  },3000);
}